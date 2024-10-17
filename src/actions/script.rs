use super::{Action, ActionError};
use crate::context::Context;
use duct::Expression;
use duct_sh::sh_dangerous;
use log::{debug, error, info, trace};
use std::io::{BufRead, BufReader};
use thiserror::Error;

const ACTION_NAME: &str = "SCRIPT";

/// An action to run a custom shell script.
///
/// The passed script is running in a subshell (`/bin/sh` on *nix, `cmd.exe` on Windows).
/// so it can use any feature in these shells: variable expansion, pipes, redirection.
/// Both the stdout and stderr will be captured and logged. If the script fails,
/// the failure will also be logged.
pub struct ScriptAction {
    directory: String,
    command: String,
    script: Expression,
}

/// Custom error describing the error cases for the ScriptAction.
#[derive(Debug, Error)]
pub enum ScriptError {
    /// The underlying Rust command creation failed. The parameter contains the error.
    #[error("the script cannot run: {0}")]
    ScriptFailure(#[from] std::io::Error),
    /// The script returned a non-zero exit code, usually meaning it failed to start
    /// or encountered an error. The parameters are the exit code and the failed output.
    #[error("the script returned non-zero exit code {0}")]
    NonZeroExitcode(i32),
    /// This means that an error occured when trying to read from the output of the script.
    #[error("the script returned invalid output")]
    OutputFailure,
}

impl From<ScriptError> for ActionError {
    fn from(value: ScriptError) -> Self {
        match value {
            ScriptError::ScriptFailure(_)
            | ScriptError::NonZeroExitcode(_)
            | ScriptError::OutputFailure => ActionError::FailedAction(value.to_string()),
        }
    }
}

impl ScriptAction {
    /// Creates a new script to be started in the given directory.
    pub fn new(directory: String, command: String) -> Self {
        // We can run `sh_dangerous`, because it is on the user's computer.
        let script = sh_dangerous(&command)
            .env("CI", "true")
            .env("GW_ACTION_NAME", ACTION_NAME)
            .env("GW_DIRECTORY", &directory)
            .stderr_to_stdout()
            .stdout_capture()
            .dir(&directory)
            .unchecked();

        ScriptAction {
            directory,
            command,
            script,
        }
    }

    fn run_inner(&self, context: &Context) -> Result<(), ScriptError> {
        // We can run `sh_dangerous`, because it is on the user's computer.
        let mut script = self.script.clone();

        // Set the environment variables
        for (key, value) in context {
            script = script.env(format!("GW_{key}"), value);
        }

        // Start the shell script
        let child = script.reader()?;

        let mut reader = BufReader::new(&child).lines();
        trace!("Reading lines from the script.");
        let command_id = self.command.as_str();
        while let Some(Ok(line)) = reader.next() {
            debug!("[{command_id}] {line}");
        }

        if let Ok(Some(output)) = child.try_wait() {
            if output.status.success() {
                Ok(())
            } else {
                Err(ScriptError::NonZeroExitcode(
                    output.status.code().unwrap_or(-1),
                ))
            }
        } else {
            Err(ScriptError::OutputFailure)
        }
    }
}

impl Action for ScriptAction {
    /// Run the script in a subshell (`/bin/sh` on *nix, `cmd.exe` on Windows).
    /// If the script fails to start, return a non-zero error code or prints non-utf8
    /// characters, this function will result in an error.
    fn run(&mut self, context: &Context) -> Result<(), ActionError> {
        debug!(
            "Running script: {} in directory {}.",
            self.command, self.directory
        );

        match self.run_inner(context) {
            Ok(()) => {
                info!("Script {:?} finished successfully.", self.command);
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
    use std::collections::HashMap;

    fn validate_output<F>(command: &str, asserter: F)
    where
        F: Fn(Vec<&str>),
    {
        let command = format!("[{command}] ");
        testing_logger::validate(|captured_logs| {
            let output: Vec<&str> = captured_logs
                .iter()
                .filter_map(|line| {
                    if line.body.starts_with(&command) {
                        Some(line.body.as_str().trim_start_matches(&command))
                    } else {
                        None
                    }
                })
                .collect();

            asserter(output);
        });
    }

    #[test]
    fn it_should_create_new_script() {
        let action = ScriptAction::new(String::from("."), String::from("echo test"));

        assert_eq!("echo test", action.command);
        assert_eq!(".", action.directory);
    }

    #[test]
    fn it_should_run_the_script() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = "echo test";
        let action = ScriptAction::new(String::from("."), String::from(command));

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output(command, |lines| {
            assert_eq!(vec!["test"], lines);
        });

        Ok(())
    }

    #[test]
    fn it_should_set_the_env_vars() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = "printenv";
        let action = ScriptAction::new(String::from("."), String::from(command));

        let context: Context = HashMap::from([
            ("TRIGGER_NAME", "TEST-TRIGGER".to_string()),
            ("CHECK_NAME", "TEST-CHECK".to_string()),
        ]);
        action.run_inner(&context)?;

        validate_output(command, |lines| {
            dbg!(&lines);
            assert!(lines.contains(&"CI=true"));
            assert!(lines.contains(&"GW_TRIGGER_NAME=TEST-TRIGGER"));
            assert!(lines.contains(&"GW_CHECK_NAME=TEST-CHECK"));
            assert!(lines.contains(&"GW_ACTION_NAME=SCRIPT"));
            assert!(lines.contains(&"GW_DIRECTORY=."));
        });

        Ok(())
    }

    #[test]
    fn it_should_keep_the_already_set_env_vars() -> Result<(), ScriptError> {
        testing_logger::setup();

        std::env::set_var("GW_TEST", "GW_TEST");

        let command = "printenv";
        let action = ScriptAction::new(String::from("."), String::from("printenv"));

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output(command, |lines| {
            assert!(lines.contains(&"GW_TEST=GW_TEST"));
        });

        Ok(())
    }

    #[test]
    fn it_should_catch_error_output() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = "echo err >&2";
        let action = ScriptAction::new(String::from("."), String::from(command));

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output(command, |lines| {
            assert_eq!(vec!["err"], lines);
        });

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_fails() -> Result<(), ScriptError> {
        let action = ScriptAction::new(String::from("."), String::from("false"));

        let context: Context = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::NonZeroExitcode(1))),
            "{result:?} should match non zero exit code"
        );

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_returns_non_utf8() -> Result<(), ScriptError> {
        let action =
            ScriptAction::new(String::from("."), String::from("/bin/echo -e '\\xc3\\x28'"));

        let context: Context = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::OutputFailure)),
            "{result:?} should match non output failure"
        );

        Ok(())
    }
}
