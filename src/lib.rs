mod config;
mod log;
mod schemas;
mod utils;
mod validation_error;
mod validation_state;

use config::{ActionType, RunConfig};
use validation_error::ValidationError;
use validation_state::ValidationState;

pub use crate::config::CliConfig;
use crate::schemas::{validate_as_action, validate_as_workflow};
#[cfg(not(feature = "js"))]
use glob::glob;
use serde_json::{Map, Value};

#[cfg(feature = "js")]
mod js {
    use crate::{
        config::{ActionType, JsConfig},
        utils::set_panic_hook,
    };
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_name = validateAction)]
    pub fn validate_action(src: &str) -> JsValue {
        set_panic_hook();

        let config = JsConfig {
            action_type: ActionType::Action,
            src,
            verbose: false,
        };

        run(&config)
    }

    #[wasm_bindgen(js_name = validateWorkflow)]
    pub fn validate_workflow(src: &str) -> JsValue {
        set_panic_hook();

        let config = JsConfig {
            action_type: ActionType::Workflow,
            src,
            verbose: false,
        };

        run(&config)
    }

    fn run(config: &JsConfig) -> JsValue {
        let run_config = config.into();
        let state = crate::run(&run_config);
        serde_wasm_bindgen::to_value(&state).unwrap()
    }
}

#[cfg(not(feature = "js"))]
pub mod cli {
    use std::fs;

    use crate::{
        config::{ActionType, RunConfig},
        CliConfig,
    };

    pub enum RunResult {
        Success,
        Failure,
    }

    pub fn run(config: &CliConfig) -> RunResult {
        let mut success = true;

        for path in &config.src {
            let file_name = match path.file_name() {
                Some(file_name) => file_name.to_str(),
                None => {
                    eprintln!("Unable to derive file name from src!");
                    success = false;
                    continue;
                }
            };

            let src = &match fs::read_to_string(path) {
                Ok(src) => src,
                Err(err) => {
                    eprintln!("Unable to read file: {err}");
                    success = false;
                    continue;
                }
            };

            let config = RunConfig {
                file_path: Some(path.to_str().unwrap()),
                file_name,
                action_type: match file_name {
                    Some("action.yml") | Some("action.yaml") => ActionType::Action,
                    _ => ActionType::Workflow,
                },
                src,
                verbose: config.verbose,
            };

            let state = crate::run(&config);

            if !state.is_valid() {
                let fmt_state = format!("{state:#?}");
                let path = state.file_path.unwrap_or("file".into());
                println!("Fatal error validating {path}");
                eprintln!("Validation failed: {fmt_state}");
                success = false;
            }
        }

        if success {
            RunResult::Success
        } else {
            RunResult::Failure
        }
    }
}

fn run(config: &RunConfig) -> ValidationState {
    let file_name = config.file_name.unwrap_or("file");
    let doc = serde_yaml::from_str(config.src);

    let mut state = match doc {
        Err(err) => ValidationState {
            action_type: Some(config.action_type),
            file_path: Some(file_name.to_string()),
            errors: vec![err.into()],
        },
        Ok(doc) => match config.action_type {
            ActionType::Action => {
                if config.verbose {
                    log::log(&format!("Treating {file_name} as an Action definition"));
                }
                validate_as_action(&doc)
            }
            ActionType::Workflow => {
                if config.verbose {
                    log::log(&format!("Treating {file_name} as a Workflow definition"));
                }
                // TODO: Re-enable path and job validation
                let mut state = validate_as_workflow(&doc);

                validate_paths(&doc, &mut state);
                validate_job_needs(&doc, &mut state);

                state
            }
        },
    };

    state.action_type = Some(config.action_type);
    state.file_path = config.file_path.map(|file_name| file_name.to_string());

    state
}

fn validate_paths(doc: &serde_json::Value, state: &mut ValidationState) {
    validate_globs(&doc["on"]["push"]["paths"], "/on/push/paths", state);
    validate_globs(
        &doc["on"]["push"]["paths-ignore"],
        "/on/push/paths-ignore",
        state,
    );
    validate_globs(
        &doc["on"]["pull_request"]["paths"],
        "/on/pull_request/paths",
        state,
    );
    validate_globs(
        &doc["on"]["pull_request"]["paths-ignore"],
        "/on/pull_request/paths-ignore",
        state,
    );
}

#[cfg(feature = "js")]
fn validate_globs(value: &serde_json::Value, path: &str, _: &mut ValidationState) {
    if !value.is_null() {
        log::warn(&format!(
            "WARNING: Glob validation is not yet supported. Glob at {path} will not be validated."
        ));
    }
}

#[cfg(not(feature = "js"))]
fn validate_globs(globs: &serde_json::Value, path: &str, state: &mut ValidationState) {
    if globs.is_null() {
        return;
    }

    if let Some(globs) = globs.as_array() {
        for g in globs {
            match glob(g.as_str().unwrap()) {
                Ok(res) => {
                    if res.count() == 0 {
                        state.errors.push(ValidationError::NoFilesMatchingGlob {
                            code: "glob_not_matched".into(),
                            path: path.into(),
                            title: "Glob does not match any files".into(),
                            detail: Some(format!("Glob {g} in {path} does not match any files")),
                        });
                    }
                }
                Err(e) => {
                    state.errors.push(ValidationError::InvalidGlob {
                        code: "invalid_glob".into(),
                        path: path.into(),
                        title: "Glob does not match any files".into(),
                        detail: Some(format!("Glob {g} in {path} is invalid: {e}")),
                    });
                }
            };
        }
    } else {
        unreachable!(
            "validate_globs called on globs object with invalid type: must be array or null"
        )
    }
}

fn validate_job_needs(doc: &serde_json::Value, state: &mut ValidationState) {
    fn is_invalid_dependency(jobs: &Map<String, Value>, need_str: &str) -> bool {
        !jobs.contains_key(need_str)
    }

    fn handle_unresolved_job(job_name: &String, needs_str: &str, state: &mut ValidationState) {
        state.errors.push(ValidationError::UnresolvedJob {
            code: "unresolved_job".into(),
            path: format!("/jobs/{job_name}/needs"),
            title: "Unresolved job".into(),
            detail: Some(format!("unresolved job {needs_str}")),
        });
    }

    if let Some(jobs) = doc["jobs"].as_object() {
        for (job_name, job) in jobs.iter() {
            let needs = &job["needs"];
            if let Some(needs_str) = needs.as_str() {
                if is_invalid_dependency(jobs, needs_str) {
                    handle_unresolved_job(job_name, needs_str, state);
                }
            } else if let Some(needs_array) = needs.as_array() {
                for needs_str in needs_array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter(|needs_str| is_invalid_dependency(jobs, needs_str))
                {
                    handle_unresolved_job(job_name, needs_str, state);
                }
            }
        }
    }
}
