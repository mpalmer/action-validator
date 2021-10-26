mod config;
mod schemas;

use std::fs::File;

pub use crate::config::Config;
use crate::schemas::{validate_as_action, validate_as_workflow};

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
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
        validate_as_workflow(&doc)
    };

    if valid_doc {
        Ok(())
    } else {
        Err("failed schema validation".into())
    }
}

fn parse_src(fd: std::fs::File) -> serde_yaml::Result<serde_json::Value> {
    serde_yaml::from_reader(fd)
}
