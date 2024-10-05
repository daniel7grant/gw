use super::{Action, ActionError};
use crate::context::Context;
use log::{debug, error, trace};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    ops::DerefMut,
    process::{Child, Command, Stdio},
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{self, sleep},
    time::Duration,
};
use thiserror::Error;

const ACTION_NAME: &str = "PROCESS";

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

/// Parameters for the process.
#[derive(Debug, Clone)]
#[cfg_attr(unix, allow(dead_code))]
pub struct ProcessParams {
    pub directory: String,
    pub command: String,
    pub retries: u32,
    pub stop_signal: String,
    pub stop_timeout: Duration,
}

/// Struct that can handle the lifecycle of the process with restarting etc.
#[derive(Debug)]
#[cfg_attr(unix, allow(dead_code))]
pub struct Process {
    child: Child,
    stop_signal: String,
    stop_timeout: Duration,
    retries: u32,
}

impl Process {
    fn start(params: &ProcessParams, tx: Sender<u32>) -> Result<Process, ProcessError> {
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
        let command_id = command.clone();
        let retries = params.retries;
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout).lines();
            while let Some(Ok(line)) = reader.next() {
                debug!("[{command_id}] {line}");
            }
            tx.send(retries).unwrap();
        });

        Ok(Process {
            child,
            stop_signal: params.stop_signal.clone(),
            stop_timeout: params.stop_timeout,
            retries,
        })
    }

    #[cfg(unix)]
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

    #[cfg(not(unix))]
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
    tx: Sender<u32>,
    params: ProcessParams,
    process: Arc<Mutex<Option<Process>>>,
}

impl ProcessAction {
    /// Creates a new process in the background.
    pub fn new(params: ProcessParams) -> Self {
        debug!(
            "Starting process: {} in directory {}.",
            params.command, params.directory
        );
        let (tx, rx) = channel();
        let process = Arc::new(Mutex::new(Some(
            Process::start(&params, tx.clone()).expect("Cannot start process."),
        )));

        let process_action = ProcessAction {
            params: params.clone(),
            process: process.clone(),
            tx: tx.clone(),
        };

        let max_retries = params.retries;
        thread::spawn(move || {
            for previous_retries in rx {
                let mut process_container = process.lock().unwrap();
                let mut new_params = params.clone();
                new_params.retries = previous_retries - 1;
                if new_params.retries > 0 {
                    debug!(
                        "Process {} failed, retrying ({} retries left).",
                        params.command, new_params.retries
                    );
                    sleep(Duration::from_millis(100));
                    process_container.replace(Process::start(&new_params, tx.clone()).unwrap());
                } else {
                    error!(
                        "Process {} failed more than {} times, we are not retrying anymore.",
                        params.command, max_retries,
                    );
                    process_container.take();
                }
            }
        });

        process_action
    }

    fn run_inner(&mut self) -> Result<(), ProcessError> {
        let mut process_container = self.process.lock().unwrap();
        if let Some(process) = process_container.deref_mut() {
            debug!("Restarting process.");
            process
                .stop()
                .map_err(|err| ProcessError::StopFailure(err.to_string()))?;
        } else {
            debug!("Cannot restart process, because it has already failed.");
        }

        process_container.replace(Process::start(&self.params, self.tx.clone())?);

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
    use std::{fs, time::Instant};
    use thread::sleep;

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
        action
            .process
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .stop()
            .unwrap();

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
        let first_pid = action.process.lock().unwrap().as_ref().unwrap().child.id();
        action.run_inner()?;
        let second_pid = action.process.lock().unwrap().as_ref().unwrap().child.id();
        action.process.lock().unwrap().as_mut().unwrap().stop()?;

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

    #[test]
    fn it_should_retry_the_process_if_it_exits_until_the_retry_count() -> Result<(), ProcessError> {
        let stop_timeout = Duration::from_secs(5);
        let params = ProcessParams {
            command: String::from("echo 1"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGTERM"),
            stop_timeout,
        };
        let action = ProcessAction::new(params);

        sleep(Duration::from_secs(1));

        let is_process_exited = action.process.lock().unwrap().as_ref().is_none();

        assert!(is_process_exited, "The process should exit.");

        Ok(())
    }

    #[test]
    fn it_should_reset_the_retries() -> Result<(), ProcessError> {
        let tailed_file = "./test_directories/tailed_file";
        let stop_timeout = Duration::from_secs(5);
        let params = ProcessParams {
            command: format!("tail -f {tailed_file}"),
            directory: String::from("."),
            retries: 5,
            stop_signal: String::from("SIGTERM"),
            stop_timeout,
        };

        // First time it should fail, because the file doesn't exist yet
        let mut action = ProcessAction::new(params);

        // Create the file and restart it quickly to see the retries reset
        fs::write(tailed_file, "").unwrap();
        action.run_inner()?;

        let is_process_running = action.process.lock().unwrap().as_ref().is_some();
        assert!(is_process_running, "The process should be running.");

        let retries = action.process.lock().unwrap().as_ref().unwrap().retries;
        assert_eq!(retries, 5);

        action.process.lock().unwrap().as_mut().unwrap().stop()?;
        fs::remove_file(tailed_file).unwrap();

        Ok(())
    }
}
