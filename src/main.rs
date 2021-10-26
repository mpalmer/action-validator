use std::process;
use structopt::StructOpt;
use action_validator::Config;

fn main() {
    let config = Config::from_args();

    if let Err(e) = action_validator::run(config) {
        println!("Fatal error: {}", e);
        process::exit(1);
    }
}
