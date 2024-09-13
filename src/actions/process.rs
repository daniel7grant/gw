use super::{Action, ActionError};
use crate::context::Context;
use duct::ReaderHandle;
use duct_sh::sh_dangerous;
use log::{debug, error};
use std::{
    io::{BufRead, BufReader},
    ops::Deref,
    sync::Arc,
    thread,
};
use thiserror::Error;

const ACTION_NAME: &str = "PROCESS";

/// An action to run in the background and restart a subprocess.
pub struct ProcessAction {
    directory: String,
    command: String,
    process: Arc<ReaderHandle>,
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
    pub fn new(directory: String, command: String) -> Self {
        let process =
            ProcessAction::start_process(&command, &directory).expect("Cannot start process.");

        ProcessAction {
            directory,
            command,
            process,
        }
    }

    fn start_process(command: &str, directory: &str) -> Result<Arc<ReaderHandle>, ProcessError> {
        debug!("Starting process: {command} in directory {directory}.",);

        // We can run `sh_dangerous`, because it is on the user's computer.
        let mut command = sh_dangerous(command);

        // Set the environment variables
        command = command.env("CI", "true");
        command = command.env("GW_ACTION_NAME", ACTION_NAME);
        command = command.env("GW_DIRECTORY", directory);

        // Start the shell script
        let handle = command
            .stderr_to_stdout()
            .stdout_capture()
            .dir(directory)
            .reader()
            .map_err(|err| ProcessError::StartFailure(err.to_string()))?;

        let return_handle = Arc::new(handle);
        let thread_handle = return_handle.clone();
        thread::spawn(move || {
            let mut reader = BufReader::new(thread_handle.deref()).lines();
            while let Some(Ok(line)) = reader.next() {
                debug!("  {line}");
            }
        });

        Ok(return_handle)
    }

    fn stop_process(&self) -> Result<(), ProcessError> {
        debug!("Stopping process to restart.");
        self.process
            .kill()
            .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

        Ok(())
    }

    fn run_inner(&mut self) -> Result<(), ProcessError> {
        self.stop_process()?;
        self.process = ProcessAction::start_process(&self.command, &self.directory)?;

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
        let action = ProcessAction::new(String::from("."), String::from("sleep 100"));

        assert_eq!("sleep 100", action.command);
        assert_eq!(".", action.directory);
    }

    #[test]
    fn it_should_restart_the_process_with_run_inner() -> Result<(), ProcessError> {
        let mut action = ProcessAction::new(String::from("."), String::from("sleep 100"));

        let first_pids = action.process.pids();
        action.run_inner()?;
        let second_pids = action.process.pids();

        assert_ne!(
            first_pids, second_pids,
            "First and second run should have different pids."
        );

        Ok(())
    }
}
