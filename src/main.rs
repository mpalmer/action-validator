use action_validator::Config;
use clap::Parser;
use std::process;

fn main() {
    let config = Config::parse();

    let mut has_errors = false;

    // Run the validator on each file and collect any errors
    config
        .src
        .iter()
        .map(|src| (src, action_validator::run_on_file(src, config.verbose)))
        .filter_map(|(src, result)| match result {
            Ok(_) => None,
            Err(error) => Some((src, error)),
        })
        .for_each(|(src, error)| {
            println!(
                "Fatal error validating {}: {}",
                src.to_str().unwrap(),
                error
            );
            has_errors = true;
        });

    if has_errors {
        process::exit(1);
    }
}
