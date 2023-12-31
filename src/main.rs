mod args;
use std::{
    sync::mpsc::{self, Sender},
    thread,
    thread::sleep,
    time::{Duration, Instant},
};

use args::Args;
use duration_string::DurationString;
use gumdrop::Options;
use gw_bin::{
    repository::{git::GitRepository, open_repository, Repository},
    script::command::run_command,
};
use tiny_http::{Response, Server};

fn run(repo: &mut GitRepository, scripts: &Vec<String>) -> Result<(), String> {
    println!("Checking directory: {}", repo.get_directory());

    if repo.check_for_updates()? {
        repo.pull_updates()?;
        println!("Pulled updates.");
        for script in scripts {
            let mut child = run_command(&repo, &script)?;
            // TODO: check exit code
            child.wait().map_err(|err| err.to_string())?;
        }
    } else {
        println!("No changes.");
    }

    Ok(())
}

fn schedule(tx: Sender<()>, delay: Duration) -> Result<(), String> {
    println!("Starting schedule in every {}.", DurationString::new(delay));
    loop {
        let next_check = Instant::now() + delay;
        tx.send(())
            .map_err(|_| String::from("Triggering run failed."))?;
        // TODO: handle overlaps
        let until_next_check = next_check - Instant::now();
        sleep(until_next_check);
    }
}

fn listen(tx: Sender<()>, http: String) -> Result<(), String> {
    let listener = Server::http(&http).map_err(|_| format!("Cannot start server on {http}"))?;
    println!("Listening on {http}...");
    for request in listener.incoming_requests() {
        println!("Received request on {} {}", request.method(), request.url());

        tx.send(())
            .map_err(|_| String::from("Triggering run failed."))?;

        request
            .respond(Response::from_string("OK"))
            .map_err(|_| String::from("Could not respond to request."))?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let Args {
        directory,
        scripts,
        trigger: _,
        http,
        delay,
        once,
        help: _,
    } = Args::parse_args_default_or_exit();

    let directory = directory.ok_or(String::from("You have to pass a directory argument."))?;
    let mut repo = open_repository(&directory)?;

    // If once is specified, check for updates and exit
    if once {
        run(&mut repo, &scripts)?;
        return Ok(());
    }

    // Allow triggers from multiple places with a channel
    let (tx, rx) = mpsc::channel::<()>();

    // Start threads for schedule and for HTTP if applicable
    let mut threads = vec![];
    let delay_duration: Duration = delay.into();
    if delay_duration > Duration::ZERO {
        let tx: Sender<()> = tx.clone();
        threads.push(thread::spawn(move || schedule(tx, delay_duration)));
    }
    if let Some(http) = http {
        let tx = tx.clone();
        threads.push(thread::spawn(move || listen(tx, http)));
    }

    // Wait for triggers in a loop
    while let Ok(_) = rx.recv() {
        run(&mut repo, &scripts)?;
    }

    for thread in threads {
        thread
            .join()
            .map_err(|_| String::from("Thread panicked."))??;
    }
    Ok(())
}
