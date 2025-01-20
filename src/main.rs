use args::{parse_args, ArgAction};
use gw_bin::{
    actions::{
        process::{ProcessAction, ProcessParams},
        script::ScriptAction,
        Action, ActionError,
    },
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
use log::{debug, error, warn, SetLoggerError};
use logger::init_logger;
use std::{fs, process, time::Duration};
use thiserror::Error;

mod args;
mod logger;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("You have to pass a directory to watch.")]
    MissingDirectoryArg,
    #[error("Directory {0} not found.")]
    NonExistentDirectory(String),
    #[error("You cannot start multiple processes, only add -p or -P once.")]
    MultipleProcessArgs,
    #[error("Check failed: {0}.")]
    FailedCheck(#[from] CheckError),
    #[error("Failed setting up logger with timezones.")]
    FailedLoggerTimezones,
    #[error("Failed setting up logger.")]
    FailedLogger(#[from] SetLoggerError),
    #[error(transparent)]
    FailedStart(#[from] StartError),
    #[error("Action failed: {0}.")]
    FailedAction(#[from] ActionError),
}

fn main_inner() -> Result<(), MainError> {
    let (args, arg_actions) = parse_args();

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    init_logger(&args)?;

    // Check if directory exists and convert to full path
    let directory_relative = args.directory.ok_or(MainError::MissingDirectoryArg)?;
    let directory_path = fs::canonicalize(directory_relative.clone())
        .map_err(|_| MainError::NonExistentDirectory(directory_relative.clone()))?;
    let directory = directory_path
        .to_str()
        .ok_or(MainError::NonExistentDirectory(directory_relative))?
        .to_string();

    // Setup triggers.
    let mut triggers: Vec<Box<dyn Trigger>> = vec![Box::new(SignalTrigger::new())];
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
    // TODO: update log message
    debug!("Setting up directory {directory} for GitCheck.");
    let mut git_check = GitCheck::open(&directory, args.git_known_host, args.trigger.into())?;
    if let Some(ssh_key) = args.ssh_key {
        git_check.set_auth(CredentialAuth::Ssh(ssh_key));
    }
    if let (Some(username), Some(password)) = (args.git_username, args.git_token) {
        git_check.set_auth(CredentialAuth::Https(username, password));
    }
    let mut check: Box<dyn Check> = Box::new(git_check);

    // Setup actions.
    if arg_actions
        .iter()
        .filter(|a| matches!(a, ArgAction::Process(_, _)))
        .count()
        > 1
    {
        return Err(MainError::MultipleProcessArgs);
    }
    let mut actions: Vec<Box<dyn Action>> = vec![];
    for arg_action in arg_actions {
        match arg_action {
            ArgAction::Script(script, runs_in_shell) => {
                debug!("Setting up ScriptAction {script:?} on change.");
                actions.push(Box::new(
                    ScriptAction::new(directory.clone(), script, runs_in_shell)
                        .map_err(ActionError::from)?,
                ));
            }
            ArgAction::Process(process, runs_in_shell) => {
                debug!("Setting up ProcessAction {process:?} on change.");
                let mut process_params =
                    ProcessParams::new(process, directory.clone(), runs_in_shell)
                        .map_err(ActionError::from)?;

                if let Some(retries) = args.process_retries {
                    process_params.set_retries(retries);
                }
                if let Some(ref stop_signal) = args.stop_signal {
                    process_params
                        .set_stop_signal(stop_signal.clone())
                        .map_err(ActionError::from)?;
                }
                if let Some(stop_timeout) = args.stop_timeout {
                    process_params.set_stop_timeout(stop_timeout.into());
                }

                actions.push(Box::new(
                    ProcessAction::new(process_params).map_err(ActionError::from)?,
                ));
            }
        }
    }

    if actions.is_empty() {
        warn!("There are no actions defined: we will only pull!");
    }

    // Start the main script.
    start(triggers, &mut check, &mut actions)?;
    Ok(())
}

fn main() {
    if let Err(err) = main_inner() {
        error!("{err}");
        process::exit(1);
    }
}
