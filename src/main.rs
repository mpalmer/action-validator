use action_validator::Config;
use clap::Parser;
use std::process;

fn main() {
    let config = Config::parse();

    // Run the validator on each file and collect any errors
    let n_errors = config
        .src
        .iter()
        .map(|src| (src, action_validator::run_on_file(src, config.verbose)))
        .filter_map(|(src, result)| match result {
            Ok(_) => None,
            Err(error) => Some((src, error)),
        })
        .map(|(src, error)| {
            println!(
                "Fatal error validating {}: {}",
                src.to_str().unwrap(),
                error
            );
        })
        .count();

    if n_errors > 0 {
        process::exit(1);
    }
}
