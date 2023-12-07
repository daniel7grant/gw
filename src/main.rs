mod args;
use std::{thread::sleep, time::Instant};

use args::Args;
use clap::Parser;

use gw::{
    repository::{git::GitRepository, open_repository, Repository},
    script::command::run_command,
};

fn run(repo: &mut GitRepository, scripts: &Vec<String>) -> Result<(), String> {
    println!("Checking directory: {}", repo.get_directory());

    if repo.check_for_updates()? {
        repo.pull_updates()?;
        println!("Pulled updates.");
        for script in scripts {
            let mut child = run_command(&repo, &script)?;
            child.wait().map_err(|err| err.to_string())?;
        }
    } else {
        println!("No changes.");
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    match args.command {
        args::Command::Run { directory, scripts } => {
            let mut repo = open_repository(&directory)?;
            run(&mut repo, &scripts)?;
            Ok(())
        }
        args::Command::Watch {
            directory,
            scripts,
            trigger: _,
            http: _,
            delay,
        } => {
            let mut repo = open_repository(&directory)?;
            loop {
                let next_check = Instant::now() + delay.into();
                run(&mut repo, &scripts)?;
                // TODO: handle overlaps
                let until_next_check = next_check - Instant::now();
                sleep(until_next_check);
            }
        }
    }
}
