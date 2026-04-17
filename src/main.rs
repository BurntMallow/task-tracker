use std::{env, process};

mod config;
mod task;
use config::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = match Config::build(args) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    if let Err(e) = config.command.execute() {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
