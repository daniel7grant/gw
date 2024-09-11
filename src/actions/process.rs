use super::{Action, ActionError};
use crate::context::Context;
use duct::Handle;
use duct_sh::sh_dangerous;
use log::{debug, error};
use std::cell::RefCell;
use thiserror::Error;

const ACTION_NAME: &str = "PROCESS";

/// An action to run in the background and restart a subprocess.
pub struct ProcessAction {
    directory: String,
    command: String,
    process: RefCell<Handle>,
}

/// Custom error describing the error cases for the ProcessAction.
#[derive(Debug, Error)]
pub enum ProcessError {
    /// The underlying Rust command creation failed. The parameter contains the error.
    #[error("the script cannot run: {0}")]
    ProcessFailure(#[from] std::io::Error),
}

impl From<ProcessError> for ActionError {
    fn from(value: ProcessError) -> Self {
        ActionError::FailedAction(value.to_string())
    }
}

impl ProcessAction {
    /// Creates a new process on a new thread.
    pub fn new(directory: String, command: String) -> Self {
        let process = RefCell::new(
            ProcessAction::start_process(&command, &directory).expect("Cannot start process."),
        );

        ProcessAction {
            directory,
            command,
            process,
        }
    }

    fn start_process(command: &str, directory: &str) -> Result<Handle, ProcessError> {
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
            .unchecked()
            .start()?;

        Ok(handle)
    }

    fn stop_process(&self) -> Result<(), ProcessError> {
        self.process.borrow().kill()?;
        Ok(())
    }

    fn run_inner(&self) -> Result<(), ProcessError> {
        self.stop_process()?;
        self.process.replace(ProcessAction::start_process(
            &self.command,
            &self.directory,
        )?);

        Ok(())
    }
}

impl Action for ProcessAction {
    /// Kills and restarts the subprocess.
    fn run(&self, _context: &Context) -> Result<(), ActionError> {
        debug!(
            "Running script: {} in directory {}.",
            self.command, self.directory
        );

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
    fn it_should_create_new_script() {
        let action = ProcessAction::new(String::from("."), String::from("echo test"));

        assert_eq!("echo test", action.command);
        assert_eq!(".", action.directory);
    }
}
