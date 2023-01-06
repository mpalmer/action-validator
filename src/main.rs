use action_validator::Config;
use clap::Parser;
use std::process;

fn main() {
    let config = Config::parse();

    if let Err(e) = action_validator::run(&config) {
        println!(
            "Fatal error validating {}: {}",
            config.src.to_str().unwrap(),
            e
        );
        process::exit(1);
    }
}
