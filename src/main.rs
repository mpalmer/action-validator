use action_validator::Config;
use std::process;
use structopt::StructOpt;

fn main() {
    let config = Config::from_args();

    if let Err(e) = action_validator::run(&config) {
        println!(
            "Fatal error validating {}: {}",
            config.src.to_str().unwrap(),
            e
        );
        process::exit(1);
    }
}
