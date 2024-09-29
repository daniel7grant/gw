use super::{Action, ActionError};
use crate::context::Context;
use log::{debug, error, trace};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};
use thiserror::Error;

const ACTION_NAME: &str = "PROCESS";

/// Parameters for the process.
#[derive(Debug)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct ProcessParams {
    pub directory: String,
    pub command: String,
    pub retries: u32,
    pub stop_signal: String,
    pub stop_timeout: Duration,
}

/// Struct that can handle the lifecycle of the process with restarting etc.
#[derive(Debug)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct Process {
    child: Child,
    stop_signal: String,
    stop_timeout: Duration,
}

impl Process {
    fn start(params: &ProcessParams) -> Result<Process, ProcessError> {
        let split_args = shlex::split(&params.command)
            .ok_or(ProcessError::StopFailure(params.command.clone()))?;

        let (command, args) = split_args
            .split_first()
            .ok_or(ProcessError::StopFailure(params.command.clone()))?;

        trace!("Running command {} with args: {:?}", command, args);

        // Set the environment variables
        let vars: HashMap<&str, &str> = HashMap::from_iter(vec![
            ("CI", "true"),
            ("GW_ACTION_NAME", ACTION_NAME),
            ("GW_DIRECTORY", &params.directory),
        ]);

        // Start the shell script
        let mut child = Command::new(command.clone())
            .envs(vars)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&params.directory)
            .spawn()
            .map_err(|err| ProcessError::StartFailure(err.to_string()))?;

        let stdout = child.stdout.take().unwrap();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout).lines();
            while let Some(Ok(line)) = reader.next() {
                debug!("  {line}");
            }
        });

        Ok(Process {
            child,
            stop_signal: params.stop_signal.clone(),
            stop_timeout: params.stop_timeout,
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn stop(&mut self) -> Result<(), ProcessError> {
        use duration_string::DurationString;
        use log::trace;
        use nix::sys::signal::kill;
        use nix::{sys::signal::Signal, unistd::Pid};
        use std::time::Instant;
        use std::{str::FromStr, thread::sleep};

        let signal = Signal::from_str(&self.stop_signal).unwrap();
        let pid = Pid::from_raw(self.child.id() as i32);

        trace!("Trying to stop process: sending {} to {}.", signal, pid);
        kill(pid, signal).unwrap();

        let start_time = Instant::now();
        while start_time.elapsed() < self.stop_timeout {
            trace!("Testing process state.");
            if let Ok(Some(output)) = self.child.try_wait() {
                debug!("Process stopped gracefully with status {}.", output);
                return Ok(());
            }
            sleep(Duration::from_secs(1));
        }

        debug!(
            "Process didn't stop gracefully after {}. Killing process.",
            DurationString::from(self.stop_timeout).to_string()
        );

        self.child
            .kill()
            .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

        debug!("Process killed successfully.");

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn stop(&mut self) -> Result<(), ProcessError> {
        self.handle
            .kill()
            .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

        debug!("Process stopped successfully.");

        Ok(())
    }
}

/// An action to run in the background and restart a subprocess.
#[derive(Debug)]
pub struct ProcessAction {
    params: ProcessParams,
    process: Process,
}

/// Custom error describing the error cases for the ProcessAction.
#[derive(Debug, Error)]
pub enum ProcessError {
    /// The underlying Rust command creation failed. The parameter contains the error.
    #[error("the command {0} cannot be parsed")]
    CommandParseFailure(String),
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
    use std::time::Instant;

    #[test]
    fn it_should_start_a_new_process() {
        let params = ProcessParams {
            command: String::from("sleep 100"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGTERM"),
            stop_timeout: Duration::from_secs(5),
        };
        let mut action = ProcessAction::new(params);
        action.process.stop().unwrap();

        assert_eq!("sleep 100", action.params.command);
        assert_eq!(".", action.params.directory);
    }

    #[test]
    fn it_should_restart_the_process_gracefully() -> Result<(), ProcessError> {
        let stop_timeout = Duration::from_secs(5);
        let params = ProcessParams {
            command: String::from("sleep 100"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGTERM"),
            stop_timeout,
        };
        let mut action = ProcessAction::new(params);

        let initial_time = Instant::now();
        let first_pid = action.process.child.id();
        action.run_inner()?;
        let second_pid = action.process.child.id();
        action.process.stop()?;

        assert_ne!(
            first_pid, second_pid,
            "First and second run should have different pids."
        );
        assert!(
            initial_time.elapsed() <= stop_timeout,
            "The stop timeout should not be elapsed."
        );

        Ok(())
    }
}
