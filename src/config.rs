use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "action-validator", about = "A validator for GitHub Action and Workflow YAML files")]
pub struct Config {
    /// Be more verbose
    #[structopt(short, long)]
    pub verbose: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    pub src: PathBuf,
}
