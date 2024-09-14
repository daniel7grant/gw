use super::{Action, ActionError};
use crate::context::Context;
use duct::ReaderHandle;
use duct_sh::sh_dangerous;
use log::{debug, error};
use std::{
    io::{BufRead, BufReader},
    sync::Arc,
    thread,
    time::Duration,
};
use thiserror::Error;

const ACTION_NAME: &str = "PROCESS";

/// Parameters for the process.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ProcessParams {
    pub directory: String,
    pub command: String,
    pub retries: u32,
    pub stop_signal: String,
    pub stop_timeout: Duration,
}

/// Struct that can handle the lifecycle of the process with restarting etc.
#[derive(Debug)]
pub struct Process {
    handle: ReaderHandle,
}

impl Process {
    fn start(params: &ProcessParams) -> Result<Arc<Process>, ProcessError> {
        // We can run `sh_dangerous`, because it is on the user's computer.
        let mut command = sh_dangerous(&params.command);

        // Set the environment variables
        command = command.env("CI", "true");
        command = command.env("GW_ACTION_NAME", ACTION_NAME);
        command = command.env("GW_DIRECTORY", &params.directory);

        // Start the shell script
        let handle = command
            .stderr_to_stdout()
            .stdout_capture()
            .dir(&params.directory)
            .reader()
            .map_err(|err| ProcessError::StartFailure(err.to_string()))?;

        let process = Arc::new(Process {
            handle,
        });
        let thread_process = process.clone();
        thread::spawn(move || {
            let thread_handle = &thread_process.handle;
            let mut reader = BufReader::new(thread_handle).lines();
            while let Some(Ok(line)) = reader.next() {
                debug!("  {line}");
            }
        });

        Ok(process)
    }

    fn stop(&self) -> Result<(), ProcessError> {
        self.handle
            .kill()
            .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

        Ok(())
    }
}

/// An action to run in the background and restart a subprocess.
#[derive(Debug)]
pub struct ProcessAction {
    params: ProcessParams,
    process: Arc<Process>,
}

/// Custom error describing the error cases for the ProcessAction.
#[derive(Debug, Error)]
pub enum ProcessError {
    /// The underlying Rust command creation failed. The parameter contains the error.
    #[error("the script cannot start: {0}")]
    StartFailure(String),
    /// Killing the command failed.
    #[error("the script cannot be stopped: {0}")]
    StopFailure(String),
}

impl From<ProcessError> for ActionError {
    fn from(value: ProcessError) -> Self {
        ActionError::FailedAction(value.to_string())
    }
}

impl ProcessAction {
    /// Creates a new process in the background.
    pub fn new(params: ProcessParams) -> Self {
        debug!(
            "Starting process: {} in directory {}.",
            params.command, params.directory
        );
        let process = Process::start(&params).expect("Cannot start process.");

        ProcessAction { params, process }
    }

    fn run_inner(&mut self) -> Result<(), ProcessError> {
        debug!("Restarting process.");
        self.process
            .stop()
            .map_err(|err| ProcessError::StopFailure(err.to_string()))?;
        self.process = Process::start(&self.params)?;

        Ok(())
    }
}

impl Action for ProcessAction {
    /// Kills and restarts the subprocess.
    fn run(&mut self, _context: &Context) -> Result<(), ActionError> {
        match self.run_inner() {
            Ok(()) => {
                debug!("Process restarted.");
                Ok(())
            }
            Err(err) => {
                error!("Failed: {err}.");
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_start_a_new_process() {
        let params = ProcessParams {
            command: String::from("sleep 100"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGINT"),
            stop_timeout: Duration::from_secs(5),
        };
        let action = ProcessAction::new(params);

        assert_eq!("sleep 100", action.params.command);
        assert_eq!(".", action.params.directory);
    }

    #[test]
    fn it_should_restart_the_process_with_run_inner() -> Result<(), ProcessError> {
        let params = ProcessParams {
            command: String::from("sleep 100"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGINT"),
            stop_timeout: Duration::from_secs(5),
        };
        let mut action = ProcessAction::new(params);

        let first_pids = action.process.handle.pids();
        action.run_inner()?;
        let second_pids = action.process.handle.pids();

        assert_ne!(
            first_pids, second_pids,
            "First and second run should have different pids."
        );

        Ok(())
    }
}
