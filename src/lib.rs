mod config;
mod schemas;

use std::fs::File;

pub use crate::config::Config;
use crate::schemas::{validate_as_action, validate_as_workflow};
use glob::glob;

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
        validate_as_workflow(&doc) && validate_paths(&doc)
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
                        eprintln!("Glob {} in {} does not match any files", g, path);
                        success = false;
                    }
                }
                Err(e) => {
                    eprintln!("Glob {} in {} is invalid: {}", g, path, e);
                }
            };
        }

        success
    }
}
