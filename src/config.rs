use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "action-validator",
    about = "A validator for GitHub Action and Workflow YAML files"
)]
pub struct Config {
    /// Be more verbose
    #[structopt(short, long)]
    pub verbose: bool,

    /// Input file
    #[structopt(parse(from_os_str), name = "path_to_action_yaml")]
    pub src: PathBuf,
}
