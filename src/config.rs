use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "action-validator",
    about = "A validator for GitHub Action and Workflow YAML files",
    version
)]
pub struct CliConfig {
    /// Be more verbose
    #[arg(short, long)]
    pub verbose: bool,

    /// Perform remote calls to validate actions exist
    #[arg(short, long)]
    pub remote_checks: bool,

    /// Input file
    #[arg(name = "path_to_action_yaml")]
    pub src: Vec<PathBuf>,
}

#[derive(Serialize, Copy, Clone, Debug)]
pub enum ActionType {
    #[serde(rename = "action")]
    Action,
    #[serde(rename = "workflow")]
    Workflow,
}

pub struct JsConfig<'a> {
    pub action_type: ActionType,
    pub src: &'a str,
    pub verbose: bool,
}

pub struct RunConfig<'a> {
    pub file_path: Option<&'a str>,
    pub file_name: Option<&'a str>,
    pub action_type: ActionType,
    pub src: &'a str,
    pub verbose: bool,
}

impl<'a> From<&JsConfig<'a>> for RunConfig<'a> {
    fn from(config: &JsConfig<'a>) -> Self {
        RunConfig {
            file_path: None,
            file_name: None,
            action_type: config.action_type,
            src: config.src,
            verbose: config.verbose,
        }
    }
}
