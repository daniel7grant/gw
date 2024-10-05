use super::{Action, ActionError};
use crate::context::Context;
use log::{debug, error, trace};
use nix::{errno::Errno, sys::signal::Signal};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    str::FromStr,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use thiserror::Error;

const ACTION_NAME: &str = "PROCESS";

/// Custom error describing the error cases for the ProcessAction.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProcessError {
    /// The command is invalid (usually mismatched quotations etc.).
    #[error("the command {0:?} cannot be parsed")]
    CommandParseFailure(String),
    /// Signal is not a valid UNIX signal.
    #[error("the signal {0} is not valid")]
    SignalParseFailure(String),
    /// The underlying Rust command creation failed. The parameter contains the error.
    #[error("the script cannot start: {0}")]
    StartFailure(String),
    /// Stopping the command failed.
    #[error("the script cannot be stopped: {0}")]
    StopFailure(String),
    /// Killing the command failed.
    #[error("killing the process failed with error: {0}")]
    KillFailed(#[from] Errno),
    /// The lock on the child is poisoned: this means the thread failed while holding the lock.
    #[error("the mutex is poisoned")]
    MutexPoisoned,
}

impl From<ProcessError> for ActionError {
    fn from(value: ProcessError) -> Self {
        match value {
            ProcessError::CommandParseFailure(_) | ProcessError::SignalParseFailure(_) => {
                ActionError::Misconfigured(value.to_string())
            }
            ProcessError::StartFailure(_)
            | ProcessError::StopFailure(_)
            | ProcessError::KillFailed(_)
            | ProcessError::MutexPoisoned => ActionError::FailedAction(value.to_string()),
        }
    }
}

/// Parameters for the process.
#[derive(Debug, Clone)]
#[cfg_attr(unix, allow(dead_code))]
pub struct ProcessParams {
    directory: String,
    command: String,
    args: Vec<String>,
    retries: u32,
    stop_signal: Signal,
    stop_timeout: Duration,
}

impl ProcessParams {
    pub fn new(original_command: String, directory: String) -> Result<ProcessParams, ProcessError> {
        let split_args = shlex::split(&original_command)
            .ok_or(ProcessError::CommandParseFailure(original_command.clone()))?;

        let (command, args) = split_args
            .split_first()
            .ok_or(ProcessError::CommandParseFailure(original_command.clone()))?;

        trace!(
            "Parsing {:?} to command {:?} and args {:?}.",
            &original_command,
            &command,
            &args
        );

        Ok(ProcessParams {
            directory,
            command: command.clone(),
            args: args.to_vec(),
            retries: 5,
            stop_signal: Signal::SIGTERM,
            stop_timeout: Duration::from_secs(1),
        })
    }

    pub fn set_retries(&mut self, retries: u32) {
        self.retries = retries;
    }

    pub fn set_stop_signal(mut self, stop_signal: String) -> Result<(), ProcessError> {
        self.stop_signal = Signal::from_str(&stop_signal)
            .map_err(|_| ProcessError::SignalParseFailure(stop_signal))?;

        Ok(())
    }

    pub fn set_stop_timeout(mut self, stop_timeout: Duration) {
        self.stop_timeout = stop_timeout;
    }
}

/// Struct that can handle the lifecycle of the process with restarting etc.
#[derive(Debug)]
#[cfg_attr(unix, allow(dead_code))]
pub struct Process {
    child: Arc<Mutex<Option<Child>>>,
    stop_signal: Signal,
    stop_timeout: Duration,
}

impl Process {
    fn start_child(params: &ProcessParams) -> Result<Child, ProcessError> {
        trace!(
            "Running command {:?} with args: {:?}.",
            params.command,
            params.args
        );

        // Set the environment variables
        let vars: HashMap<&str, &str> = HashMap::from_iter(vec![
            ("CI", "true"),
            ("GW_ACTION_NAME", ACTION_NAME),
            ("GW_DIRECTORY", &params.directory),
        ]);

        // Start the command
        let child = Command::new(&params.command)
            .envs(vars)
            .args(&params.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&params.directory)
            .spawn()
            .map_err(|err| ProcessError::StartFailure(err.to_string()))?;

        Ok(child)
    }

    fn start(params: &ProcessParams) -> Result<Process, ProcessError> {
        let child = Arc::new(Mutex::new(Some(Process::start_child(params)?)));

        let command_id = params.command.clone();
        let max_retries = params.retries;
        let thread_params = params.clone();
        let thread_child = child.clone();
        thread::spawn(move || {
            let mut retries = max_retries;

            while retries > 0 {
                if let Some(stdout) = thread_child
                    .lock()
                    .ok()
                    .and_then(|mut child| child.as_mut().and_then(|child| child.stdout.take()))
                {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Some(Ok(line)) = reader.next() {
                        debug!("[{command_id}] {line}");
                    }

                    debug!(
                        "Process {:?} failed, retrying ({} retries left).",
                        thread_params.command, retries
                    );

                    retries -= 1;
                    sleep(Duration::from_millis(100));
                    match Process::start_child(&thread_params) {
                        Ok(new_child) => {
                            if let Ok(mut unlocked_child) = thread_child.lock() {
                                unlocked_child.replace(new_child);
                            } else {
                                error!("Failed locking the child, the mutex might be poisoned.");
                            }
                        }
                        Err(err) => {
                            error!("Failed retrying the process: {err}.");
                            break;
                        }
                    }
                } else {
                    error!("Failed taking the stdout of process.");
                    break;
                }
            }

            if let Ok(mut unlocked_child) = thread_child.lock() {
                unlocked_child.take();
            } else {
                error!("Failed locking the child, the mutex might be poisoned.");
            }

            error!(
                "Process {:?} failed more than {} times, we are not retrying anymore.",
                thread_params.command, max_retries,
            );
        });

        Ok(Process {
            child,
            stop_signal: params.stop_signal,
            stop_timeout: params.stop_timeout,
        })
    }

    #[cfg(not(unix))]
    fn stop(&mut self) -> Result<(), ProcessError> {
        use duration_string::DurationString;
        use log::trace;
        use nix::sys::signal::kill;
        use nix::unistd::Pid;
        use std::thread::sleep;
        use std::time::Instant;

        if let Some(child) = self
            .child
            .lock()
            .map_err(|_| ProcessError::MutexPoisoned)?
            .as_mut()
        {
            let pid = Pid::from_raw(child.id() as i32);

            trace!(
                "Trying to stop process: sending {} to {}.",
                self.stop_signal,
                pid
            );
            kill(pid, self.stop_signal)?;

            let start_time = Instant::now();
            while start_time.elapsed() < self.stop_timeout {
                trace!("Testing process state.");
                if let Ok(Some(output)) = child.try_wait() {
                    debug!("Process stopped gracefully with status {}.", output);
                    return Ok(());
                }
                sleep(Duration::from_secs(1));
            }

            debug!(
                "Process didn't stop gracefully after {}. Killing process.",
                DurationString::from(self.stop_timeout).to_string()
            );

            child
                .kill()
                .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

            debug!("Process killed successfully.");
        } else {
            debug!("Cannot restart process, because it has already failed.");
        }

        Ok(())
    }

    #[cfg(unix)]
    fn stop(&mut self) -> Result<(), ProcessError> {
        if let Some(child) = self
            .child
            .lock()
            .map_err(|_| ProcessError::MutexPoisoned)?
            .as_mut()
        {
            child
                .kill()
                .map_err(|err| ProcessError::StopFailure(err.to_string()))?;

            debug!("Process stopped successfully.");
        } else {
            debug!("Cannot restart process, because it has already failed.");
        }

        Ok(())
    }
}

/// An action to run in the background and restart a subprocess.
#[derive(Debug)]
pub struct ProcessAction {
    params: ProcessParams,
    process: Process,
}

impl ProcessAction {
    /// Creates a new process in the background.
    pub fn new(params: ProcessParams) -> Result<ProcessAction, ProcessError> {
        debug!(
            "Starting process: {:?} in directory {}.",
            params.command, params.directory
        );
        let process = Process::start(&params)?;

        Ok(ProcessAction { params, process })
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
    use std::{fs, time::Instant};
    use thread::sleep;

    #[test]
    fn it_should_start_a_new_process() -> Result<(), ProcessError> {
        let params = ProcessParams::new(String::from("sleep 100"), String::from("."))?;
        let mut action = ProcessAction::new(params)?;
        action.process.stop()?;

        assert_eq!("sleep", action.params.command);
        assert_eq!(vec!["100"], action.params.args);
        assert_eq!(".", action.params.directory);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_command_is_invalid() -> Result<(), ProcessError> {
        let failing_command = String::from("sleep '100");
        let failing_params = ProcessParams::new(failing_command.clone(), String::from("."));

        assert_eq!(
            ProcessError::CommandParseFailure(failing_command),
            failing_params.unwrap_err(),
        );

        Ok(())
    }

    #[test]
    fn it_should_fail_if_signal_is_invalid() -> Result<(), ProcessError> {
        let failing_signal = String::from("SIGWTF");
        let failing_params = ProcessParams::new(String::from("sleep 100"), String::from("."))?
            .set_stop_signal(failing_signal.clone());

        assert_eq!(
            ProcessError::SignalParseFailure(failing_signal),
            failing_params.unwrap_err(),
        );

        Ok(())
    }

    #[test]
    fn it_should_restart_the_process_gracefully() -> Result<(), ProcessError> {
        let stop_timeout = Duration::from_secs(5);
        let params = ProcessParams::new(String::from("sleep 100"), String::from("."))?;
        let mut action = ProcessAction::new(params)?;

        let initial_time = Instant::now();
        let first_pid = action.process.child.lock().unwrap().as_ref().unwrap().id();
        action.run_inner()?;
        let second_pid = action.process.child.lock().unwrap().as_ref().unwrap().id();
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

    #[test]
    fn it_should_retry_the_process_if_it_exits_until_the_retry_count() -> Result<(), ProcessError> {
        let params = ProcessParams::new(String::from("false"), String::from("."))?;
        let action = ProcessAction::new(params)?;

        sleep(Duration::from_secs(1));

        let is_child_exited = action.process.child.lock().unwrap().as_ref().is_none();

        assert!(is_child_exited, "The child should exit.");

        Ok(())
    }

    #[test]
    fn it_should_reset_the_retries() -> Result<(), ProcessError> {
        let tailed_file = "./test_directories/tailed_file";
        let params = ProcessParams::new(format!("tail -f {tailed_file}"), String::from("."))?;

        // First time it should fail, because the file doesn't exist yet
        let mut action = ProcessAction::new(params)?;

        // Create the file and restart it quickly to see the retries reset
        fs::write(tailed_file, "").unwrap();
        action.run_inner()?;

        let is_child_running = action.process.child.lock().unwrap().as_ref().is_some();
        assert!(is_child_running, "The child should be running.");

        action.process.stop()?;
        fs::remove_file(tailed_file).unwrap();

        Ok(())
    }
}
