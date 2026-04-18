mod config;
mod schemas;
mod system;
mod utils;
mod validation_error;
mod validation_state;

use config::{ActionType, RunConfig};
use std::path::PathBuf;
use validation_error::ValidationError;
use validation_state::ValidationState;

pub use crate::config::CliConfig;
use crate::schemas::{validate_as_action, validate_as_workflow};

#[cfg(feature = "js")]
mod js {
    use super::cli;
    use crate::config::CliConfig;
    use crate::system;
    use crate::{
        config::{ActionType, JsConfig},
        utils::set_panic_hook,
    };
    use clap::Parser as _;
    use js_sys::Array;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_name = main)]
    pub fn main(args: Array) -> JsValue {
        set_panic_hook();

        let rust_args: Vec<String> = args
            .iter()
            .map(|arg| arg.as_string().unwrap_or_default())
            .collect();

        let config = match CliConfig::try_parse_from(rust_args) {
            Ok(config) => config,
            Err(error) => {
                let error_text = if system::process::stdout::is_tty() {
                    format!("{}", error.render().ansi())
                } else {
                    error.render().to_string()
                };
                system::console::error(&error_text);
                system::process::exit(error.exit_code());
            }
        };

        if matches!(cli::run(&config), cli::RunResult::Failure) {
            system::process::exit(1);
        }

        system::process::exit(0);
    }

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

pub mod cli {
    use crate::{
        config::{ActionType, RunConfig},
        system, CliConfig, ValidationState,
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

            let src = &match system::fs::read_to_string(path) {
                Ok(src) => src,
                Err(err) => {
                    system::console::error(&format!(
                        "Unable to read file {}: {err}",
                        path.display()
                    ));
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
                rootdir: config.rootdir.clone(),
                allow_remote_checks: config.allow_remote_checks,
            };

            let state = crate::run(&config);
            let path = state.file_path.as_deref().unwrap_or("file");

            if config.verbose {
                for warning in &state.warnings {
                    system::console::warn(&format!("Warning validating {path}: {warning:?}"));
                }
            }

            if !state.is_valid() {
                system::console::log(&format!("Fatal error validating {path}"));
                // Strip warnings from the dump so stderr snapshots don't churn
                // when validation also happens to emit skipped-check warnings.
                let errors_only = ValidationState {
                    warnings: Vec::new(),
                    ..state
                };
                system::console::error(&format!("Validation failed: {errors_only:#?}"));
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
    let doc = yaml_serde::from_str(config.src);

    let mut state = match doc {
        Err(err) => ValidationState {
            action_type: Some(config.action_type),
            file_path: Some(file_name.to_string()),
            errors: vec![err.into()],
            ..Default::default()
        },
        Ok(doc) => match config.action_type {
            ActionType::Action => {
                if config.verbose {
                    system::console::log(&format!("Treating {file_name} as an Action definition"));
                }
                let mut state = validate_as_action(&doc);
                validate_uses(&doc, config, &mut state);

                state
            }
            ActionType::Workflow => {
                if config.verbose {
                    system::console::log(&format!("Treating {file_name} as a Workflow definition"));
                }
                // TODO: Re-enable path and job validation
                let mut state = validate_as_workflow(&doc);

                validate_paths(&doc, config.rootdir.as_ref(), &mut state);
                validate_job_needs(&doc, &mut state);
                validate_uses(&doc, config, &mut state);

                state
            }
        },
    };

    state.action_type = Some(config.action_type);
    state.file_path = config.file_path.map(|file_name| file_name.to_string());

    state
}

fn validate_paths(doc: &serde_json::Value, rootdir: Option<&PathBuf>, state: &mut ValidationState) {
    validate_globs(
        &doc["on"]["push"]["paths"],
        "/on/push/paths",
        rootdir,
        state,
    );
    validate_globs(
        &doc["on"]["push"]["paths-ignore"],
        "/on/push/paths-ignore",
        rootdir,
        state,
    );
    validate_globs(
        &doc["on"]["pull_request"]["paths"],
        "/on/pull_request/paths",
        rootdir,
        state,
    );
    validate_globs(
        &doc["on"]["pull_request"]["paths-ignore"],
        "/on/pull_request/paths-ignore",
        rootdir,
        state,
    );
}

fn validate_globs(
    globs: &serde_json::Value,
    path: &str,
    rootdir: Option<&PathBuf>,
    state: &mut ValidationState,
) {
    if globs.is_null() {
        return;
    }

    if let Some(globs) = globs.as_array() {
        let git_files = match system::git::ls_files() {
            Ok(files) => files,
            Err(e) => {
                state.errors.push(ValidationError::InvalidGlob {
                    code: "git_ls_files_failed".into(),
                    path: path.into(),
                    title: "Failed to get git tracked files".into(),
                    detail: Some(format!("git ls-files failed: {e}")),
                });
                return;
            }
        };

        let git_file_refs: Vec<&str> = git_files.iter().map(|s| s.as_str()).collect();

        for g in globs {
            let glob = g.as_str().expect("glob to be a string");
            let pattern = if glob.starts_with('!') {
                glob.chars().skip(1).collect()
            } else {
                glob.to_string()
            };

            let pattern = if let Some(rootdir) = rootdir {
                rootdir.join(pattern).display().to_string()
            } else {
                pattern
            };

            match compare_changes::path_matches(&pattern, &git_file_refs) {
                Ok(matched_index) => {
                    if matched_index.is_none() {
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
    fn is_invalid_dependency(
        jobs: &serde_json::Map<String, serde_json::Value>,
        need_str: &str,
    ) -> bool {
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

#[derive(Debug)]
#[allow(dead_code)] // `subpath` is captured for future composite-action work
enum UsesRef<'a> {
    GitHub {
        owner: &'a str,
        repo: &'a str,
        subpath: Option<&'a str>,
        git_ref: &'a str,
    },
    Docker {
        image: &'a str,
    },
    Local {
        path: &'a str,
    },
}

fn parse_uses(uses: &str) -> Option<UsesRef<'_>> {
    if let Some(image) = uses.strip_prefix("docker://") {
        if image.is_empty() {
            return None;
        }
        return Some(UsesRef::Docker { image });
    }

    if uses.starts_with("./") || uses == "." {
        return Some(UsesRef::Local { path: uses });
    }

    let (repo_ref, git_ref) = uses.rsplit_once('@')?;
    if git_ref.is_empty() {
        return None;
    }

    let mut segments = repo_ref.splitn(3, '/');
    let owner = segments.next()?;
    let repo = segments.next()?;
    let subpath = segments.next();

    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    if let Some(sub) = subpath {
        if sub.is_empty() {
            return None;
        }
    }

    Some(UsesRef::GitHub {
        owner,
        repo,
        subpath,
        git_ref,
    })
}

fn collect_uses<'a>(
    doc: &'a serde_json::Value,
    state: &ValidationState,
    out: &mut Vec<(String, &'a str)>,
) {
    use crate::config::ActionType;

    match state.action_type {
        Some(ActionType::Action) => {
            if let Some(steps) = doc["runs"]["steps"].as_array() {
                for (i, step) in steps.iter().enumerate() {
                    if let Some(uses) = step["uses"].as_str() {
                        out.push((format!("/runs/steps/{i}/uses"), uses));
                    }
                }
            }
        }
        Some(ActionType::Workflow) | None => {
            if let Some(jobs) = doc["jobs"].as_object() {
                for (job_name, job) in jobs.iter() {
                    if let Some(uses) = job["uses"].as_str() {
                        out.push((format!("/jobs/{job_name}/uses"), uses));
                    }
                    if let Some(steps) = job["steps"].as_array() {
                        for (i, step) in steps.iter().enumerate() {
                            if let Some(uses) = step["uses"].as_str() {
                                out.push((format!("/jobs/{job_name}/steps/{i}/uses"), uses));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn validate_uses(doc: &serde_json::Value, config: &RunConfig, state: &mut ValidationState) {
    let mut refs = Vec::new();
    collect_uses(doc, state, &mut refs);

    for (path, uses) in refs {
        let parsed = match parse_uses(uses) {
            Some(parsed) => parsed,
            None => {
                state.errors.push(ValidationError::InvalidActionFormat {
                    code: "invalid_action_format".into(),
                    path,
                    title: "Invalid `uses` value".into(),
                    detail: Some(format!(
                        "`{uses}` is not a valid uses reference. Expected `owner/repo[/path]@ref`, `docker://image`, or `./local/path`."
                    )),
                });
                continue;
            }
        };

        match parsed {
            UsesRef::Local { path: local } => {
                let candidate = match config.rootdir.as_ref() {
                    Some(root) => root.join(local),
                    None => PathBuf::from(local),
                };
                if !candidate.exists() {
                    state.errors.push(ValidationError::UnresolvedAction {
                        code: "unresolved_action".into(),
                        path,
                        title: "Local action path does not exist".into(),
                        detail: Some(format!(
                            "Local action path `{local}` was not found (looked for `{}`).",
                            candidate.display()
                        )),
                    });
                }
            }
            UsesRef::GitHub { .. } | UsesRef::Docker { .. } => {
                if config.allow_remote_checks {
                    remote_checks::check_uses(&parsed, &path, state);
                } else {
                    state.warnings.push(skipped_check(
                        &path,
                        format!(
                            "Skipped remote existence check for `{uses}`. Pass `--allow-remote-checks` to enable."
                        ),
                    ));
                }
            }
        }
    }
}

fn skipped_check(path: &str, detail: String) -> ValidationError {
    ValidationError::RemoteCheckSkipped {
        code: "remote_check_skipped".into(),
        path: path.into(),
        title: "Remote check skipped".into(),
        detail: Some(detail),
    }
}

pub(crate) mod remote_checks {
    use super::{skipped_check, UsesRef, ValidationError, ValidationState};

    const DEFAULT_GITHUB_BASE_URL: &str = "https://api.github.com";

    pub(super) fn check_uses(uses: &UsesRef, path: &str, state: &mut ValidationState) {
        match uses {
            #[cfg(not(feature = "js"))]
            UsesRef::GitHub {
                owner,
                repo,
                git_ref,
                ..
            } => check_github(owner, repo, git_ref, path, DEFAULT_GITHUB_BASE_URL, state),
            #[cfg(feature = "js")]
            UsesRef::GitHub { .. } => state.warnings.push(skipped_check(
                path,
                "Remote checks are not supported in the WASM/JS build.".into(),
            )),
            UsesRef::Docker { image } => state.warnings.push(skipped_check(
                path,
                format!("Docker image existence checks are not yet implemented (image `{image}`)."),
            )),
            UsesRef::Local { .. } => {}
        }
    }

    #[cfg(not(feature = "js"))]
    pub(crate) fn check_github(
        owner: &str,
        repo: &str,
        git_ref: &str,
        path: &str,
        base_url: &str,
        state: &mut ValidationState,
    ) {
        use std::time::Duration;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build();

        let client = match client {
            Ok(c) => c,
            Err(_) => {
                state.warnings.push(skipped_check(
                    path,
                    "Failed to construct HTTP client.".into(),
                ));
                return;
            }
        };

        let url = format!("{base_url}/repos/{owner}/{repo}/commits/{git_ref}");

        match client.head(&url).send() {
            // 2xx resolves; 401/403 mean auth-gated (assume exists).
            Ok(resp) if resp.status().as_u16() == 404 => {
                state.errors.push(ValidationError::UnresolvedAction {
                    code: "unresolved_action".into(),
                    path: path.into(),
                    title: "Remote action not found".into(),
                    detail: Some(format!(
                        "GitHub returned 404 for `{owner}/{repo}@{git_ref}`."
                    )),
                });
            }
            Ok(_) => {}
            Err(err) => state.warnings.push(skipped_check(
                path,
                format!("Network error checking `{owner}/{repo}@{git_ref}`: {err}"),
            )),
        }
    }
}

#[cfg(all(test, not(feature = "js")))]
mod uses_tests {
    use super::{parse_uses, remote_checks, UsesRef};
    use crate::validation_state::ValidationState;

    #[test]
    fn parses_github_action() {
        match parse_uses("actions/checkout@v4") {
            Some(UsesRef::GitHub {
                owner,
                repo,
                subpath,
                git_ref,
            }) => {
                assert_eq!(owner, "actions");
                assert_eq!(repo, "checkout");
                assert_eq!(subpath, None);
                assert_eq!(git_ref, "v4");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn parses_docker_image() {
        assert!(matches!(
            parse_uses("docker://alpine:3.18"),
            Some(UsesRef::Docker {
                image: "alpine:3.18"
            })
        ));
    }

    #[test]
    fn parses_local_path() {
        assert!(matches!(
            parse_uses("./.github/actions/custom"),
            Some(UsesRef::Local {
                path: "./.github/actions/custom"
            })
        ));
    }

    #[test]
    fn rejects_malformed() {
        assert!(parse_uses("actions/checkout").is_none());
        assert!(parse_uses("actions@v1").is_none());
        assert!(parse_uses("docker://").is_none());
    }

    #[test]
    fn remote_check_404_yields_unresolved_action() {
        let server = httpmock::MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(httpmock::Method::HEAD)
                .path("/repos/actions/does-not-exist/commits/v2");
            then.status(404);
        });

        let mut state = ValidationState::default();

        remote_checks::check_github(
            "actions",
            "does-not-exist",
            "v2",
            "/jobs/build/steps/0/uses",
            &server.base_url(),
            &mut state,
        );

        assert_eq!(state.errors.len(), 1, "expected one UnresolvedAction error");
        assert!(state.warnings.is_empty(), "no warnings expected on 404");
    }

    #[test]
    fn remote_check_200_is_quiet() {
        let server = httpmock::MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(httpmock::Method::HEAD)
                .path("/repos/actions/checkout/commits/v4");
            then.status(200);
        });

        let mut state = ValidationState::default();

        remote_checks::check_github(
            "actions",
            "checkout",
            "v4",
            "/jobs/build/steps/0/uses",
            &server.base_url(),
            &mut state,
        );

        assert!(state.errors.is_empty());
        assert!(state.warnings.is_empty());
    }
}
