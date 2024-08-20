use args::parse_args;
use gw_bin::{
    actions::{script::ScriptAction, Action},
    checks::{
        git::{CredentialAuth, GitCheck},
        Check, CheckError,
    },
    start::{start, StartError},
    triggers::{
        http::HttpTrigger, once::OnceTrigger, schedule::ScheduleTrigger, signal::SignalTrigger,
        Trigger,
    },
};
use log::{debug, error, warn, LevelFilter, SetLoggerError};
use simple_logger::SimpleLogger;
use std::{fs, process, time::Duration};
use thiserror::Error;

mod args;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("You have to pass a directory to watch.")]
    MissingDirectoryArg,
    #[error("Directory {0} not found.")]
    NonExistentDirectory(String),
    #[error("Check failed: {0}.")]
    FailedCheck(#[from] CheckError),
    #[error("Failed setting up logger.")]
    FailedLogger(#[from] SetLoggerError),
    #[error(transparent)]
    FailedStart(#[from] StartError),
}

fn main_inner() -> Result<(), MainError> {
    let args = parse_args();

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    SimpleLogger::new()
        .with_level(match (args.quiet, args.verbose) {
            (true, _) => LevelFilter::Error,
            (false, 0) => LevelFilter::Info,
            (false, 1) => LevelFilter::Debug,
            (false, _) => LevelFilter::Trace,
        })
        .env()
        .init()?;

    // Check if directory exists and convert to full path
    let directory_relative = args.directory.ok_or(MainError::MissingDirectoryArg)?;
    let directory_path = fs::canonicalize(directory_relative.clone())
        .map_err(|_| MainError::NonExistentDirectory(directory_relative.clone()))?;
    let directory = directory_path
        .to_str()
        .ok_or(MainError::NonExistentDirectory(directory_relative))?
        .to_string();

    // Setup triggers.
    let mut triggers: Vec<Box<dyn Trigger>> = vec![Box::new(SignalTrigger)];
    if args.once {
        debug!("Setting up OnceTrigger (this will disable all other triggers).");
        triggers.push(Box::new(OnceTrigger));
    } else {
        let duration: Duration = args.delay.into();
        if !duration.is_zero() {
            debug!("Setting up ScheduleTrigger on every {}.", args.delay);
            triggers.push(Box::new(ScheduleTrigger::new(duration)));
        }
        if let Some(http) = args.http {
            debug!("Setting up HttpTrigger on {http}.");
            triggers.push(Box::new(HttpTrigger::new(http)));
        }
    }

    // Setup check.
    debug!("Setting up directory {directory} for GitCheck.");
    let mut git_check = GitCheck::open(&directory)?;
    git_check.set_known_host(args.git_known_host)?;
    if let Some(ssh_key) = args.ssh_key {
        git_check.set_auth(CredentialAuth::Ssh(ssh_key));
    }
    if let (Some(username), Some(password)) = (args.git_username, args.git_token) {
        git_check.set_auth(CredentialAuth::Https(username, password));
    }
    let mut check: Box<dyn Check> = Box::new(git_check);

    // Setup actions.
    let mut actions: Vec<Box<dyn Action>> = vec![];
    for script in args.scripts {
        debug!("Setting up ScriptAction '{script}' on change.");
        actions.push(Box::new(ScriptAction::new(directory.clone(), script)));
    }

    if actions.is_empty() {
        warn!("There are no actions defined: we will only pull!");
    }

    // Start the main script.
    start(triggers, &mut check, &actions)?;
    Ok(())
}

fn main() {
    if let Err(err) = main_inner() {
        error!("{err}");
        process::exit(1);
    }
}
