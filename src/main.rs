use std::{env, process};

mod config;
mod task;
use config::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    match Config::build(args) {
        Ok(config) => config.command.execute(),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}
