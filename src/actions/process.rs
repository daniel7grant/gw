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
    handle: ReaderHandle,
    stop_signal: String,
    stop_timeout: Duration,
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
            .unchecked()
            .reader()
            .map_err(|err| ProcessError::StartFailure(err.to_string()))?;

        let process = Arc::new(Process {
            handle,
            stop_signal: params.stop_signal.clone(),
            stop_timeout: params.stop_timeout,
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

    #[cfg(not(target_os = "windows"))]
    fn stop(&self) -> Result<(), ProcessError> {
        use duration_string::DurationString;
        use log::trace;
        use nix::sys::signal::kill;
        use nix::{sys::signal::Signal, unistd::Pid};
        use std::time::Instant;
        use std::{str::FromStr, thread::sleep};

        let signal = Signal::from_str(&self.stop_signal).unwrap();
        let pids = self.handle.pids();
        let pid = pids.first().unwrap();
        let pid = Pid::from_raw(*pid as i32);

        trace!("Trying stopping: Sending {} to {}.", signal, pid);
        kill(pid, signal).unwrap();

        let start_time = Instant::now();
        while start_time.elapsed() < self.stop_timeout {
            if kill(pid, None).is_err() {
                let output = self.handle.try_wait().unwrap().unwrap();
                debug!("Process stopped gracefully with status {}.", output.status);
                return Ok(());
            }
            sleep(Duration::from_secs(1));
        }

        debug!(
            "Process didn't stop gracefully after {}. Killing process.",
            DurationString::from(self.stop_timeout).to_string()
        );

        self.handle
            .kill()
            .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

        debug!("Process killed successfully.");

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn stop(&self) -> Result<(), ProcessError> {
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
    use log::LevelFilter;
    use simple_logger::SimpleLogger;
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
        let action = ProcessAction::new(params);
        action.process.stop().unwrap();

        assert_eq!("sleep 100", action.params.command);
        assert_eq!(".", action.params.directory);
    }

    #[test]
    fn it_should_restart_the_process_gracefully() -> Result<(), ProcessError> {
        let params = ProcessParams {
            command: String::from("sleep 100"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGTERM"),
            stop_timeout: Duration::from_secs(5),
        };
        let mut action = ProcessAction::new(params);

        let first_pids = action.process.handle.pids();
        action.run_inner()?;
        let second_pids = action.process.handle.pids();
        action.process.stop()?;

        assert_ne!(
            first_pids, second_pids,
            "First and second run should have different pids."
        );

        Ok(())
    }

    #[test]
    fn it_should_kill_the_process_if_graceful_not_working() -> Result<(), ProcessError> {
        SimpleLogger::new()
            .with_level(LevelFilter::Trace)
            .env()
            .init()
            .unwrap();

        let stop_timeout = Duration::from_secs(5);
        let params = ProcessParams {
            command: String::from("python -c 'import signal,time; signal.signal(signal.SIGUSR2, lambda x,y: print(\"trapped\")); time.sleep(1000)'"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGUSR2"),
            stop_timeout,
        };
        let mut action = ProcessAction::new(params);

        let initial_pids = action.process.handle.pids();
        let initial_time = Instant::now();
        action.run_inner().expect("Restart failed");

        let killed_pids = action.process.handle.pids();
        action.process.stop()?;
        assert_ne!(initial_pids, killed_pids, "The process should be killed.");
        assert!(
            initial_time.elapsed() >= stop_timeout,
            "The stop timeout should be elapsed."
        );

        Ok(())
    }
}
