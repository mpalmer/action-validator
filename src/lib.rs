mod config;
mod schemas;

use std::fs::File;

pub use crate::config::Config;
use crate::schemas::{validate_as_action, validate_as_workflow};
use glob::glob;
use serde_json::{Map, Value};

pub fn run(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let fd = File::open(&config.src)?;
    let doc = parse_src(fd)?;

    let file_name = config
        .src
        .file_name()
        .ok_or("Unable to derive file name from src!")?;
    let valid_doc = if file_name == "action.yml" || file_name == "action.yaml" {
        if config.verbose {
            eprintln!(
                "Treating {} as an Action definition",
                config
                    .src
                    .to_str()
                    .ok_or("Unable to convert PathBuf to string!")?
            );
        }
        validate_as_action(&doc)
    } else {
        if config.verbose {
            eprintln!(
                "Treating {} as a Workflow definition",
                config
                    .src
                    .to_str()
                    .ok_or("Unable to convert PathBuf to string!")?
            );
        }
        validate_as_workflow(&doc) && validate_paths(&doc) && validate_job_needs(&doc)
    };

    if valid_doc {
        Ok(())
    } else {
        Err("validation failed".into())
    }
}

fn parse_src(fd: std::fs::File) -> serde_yaml::Result<serde_json::Value> {
    serde_yaml::from_reader(fd)
}

fn validate_paths(doc: &serde_json::Value) -> bool {
    let mut success = true;

    success = validate_globs(&doc["on"]["push"]["paths"], "on.push.paths") && success;
    success = validate_globs(&doc["on"]["push"]["paths-ignore"], "on.push.paths-ignore") && success;
    success =
        validate_globs(&doc["on"]["pull_request"]["paths"], "on.pull_request.paths") && success;
    success = validate_globs(
        &doc["on"]["pull_request"]["paths-ignore"],
        "on.pull_request.paths-ignore",
    ) && success;

    success
}

fn validate_globs(globs: &serde_json::Value, path: &str) -> bool {
    if globs.is_null() {
        true
    } else {
        let mut success = true;

        for g in globs.as_array().unwrap() {
            match glob(g.as_str().unwrap()) {
                Ok(res) => {
                    if res.count() == 0 {
                        eprintln!("Glob {g} in {path} does not match any files");
                        success = false;
                    }
                }
                Err(e) => {
                    eprintln!("Glob {g} in {path} is invalid: {e}");
                    success = false;
                }
            };
        }

        success
    }
}

fn validate_job_needs(doc: &serde_json::Value) -> bool {
    fn is_invalid_dependency(jobs: &Map<String, Value>, need_str: &str) -> bool {
        !jobs.contains_key(need_str)
    }

    fn print_error(needs_str: &str) {
        eprintln!("unresolved job {needs_str}");
    }

    let mut success = true;
    if let Some(jobs) = doc["jobs"].as_object() {
        for job_key in jobs.keys() {
            let needs = &jobs.get(job_key).unwrap()["needs"];
            if needs.is_string() {
                let needs_str = needs.as_str().unwrap();
                if is_invalid_dependency(jobs, needs_str) {
                    success = false;
                    print_error(needs_str);
                }
            } else if needs.is_array() {
                for need in needs.as_array().unwrap() {
                    let need_str = need.as_str().unwrap();
                    if is_invalid_dependency(jobs, need_str) {
                        success = false;
                        print_error(need_str);
                    }
                }
            }
        }
    }

    success
}
