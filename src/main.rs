use std::{env, error::Error, process};

mod config;
mod persistence;
mod task;
use config::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(args)?;

    let mut tasks = persistence::load()?;
    config.command.execute(&mut tasks)?;
    persistence::save(tasks)?;
    Ok(())
}
