use super::{utils::command::create_command, Action, ActionError};
use crate::context::Context;
use duct::Expression;
use log::{debug, error, info};
use std::io::{BufRead, BufReader};
use thiserror::Error;

const ACTION_NAME: &str = "SCRIPT";

/// An action to run a custom shell script.
///
/// The passed script is running in a subshell (`/bin/sh` on *nix, `cmd.exe` on Windows).
/// so it can use any feature in these shells: variable expansion, pipes, redirection.
/// Both the stdout and stderr will be captured and logged. If the script fails,
/// the failure will also be logged.
#[derive(Debug)]
pub struct ScriptAction {
    directory: String,
    command: String,
    script: Expression,
    runs_in_shell: bool,
}

/// Custom error describing the error cases for the ScriptAction.
#[derive(Debug, Error)]
pub enum ScriptError {
    /// The command is invalid (usually mismatched quotations etc.).
    #[error("the command {0:?} cannot be parsed")]
    CommandParseFailure(String),
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
            ScriptError::CommandParseFailure(_)
            | ScriptError::ScriptFailure(_)
            | ScriptError::NonZeroExitcode(_)
            | ScriptError::OutputFailure => ActionError::FailedAction(value.to_string()),
        }
    }
}

impl ScriptAction {
    /// Creates a new script to be started in the given directory.
    pub fn new(
        directory: String,
        original_command: String,
        runs_in_shell: bool,
    ) -> Result<Self, ScriptError> {
        let (command, script) = create_command(&original_command, runs_in_shell)
            .ok_or(ScriptError::CommandParseFailure(original_command))?;

        let script = script
            .env("CI", "true")
            .env("GW_ACTION_NAME", ACTION_NAME)
            .env("GW_DIRECTORY", &directory)
            .stderr_to_stdout()
            .stdout_capture()
            .dir(&directory)
            .unchecked();

        Ok(ScriptAction {
            directory,
            command,
            script,
            runs_in_shell,
        })
    }

    fn run_inner(&self, context: &Context) -> Result<(), ScriptError> {
        // We can run `sh_dangerous`, because it is on the user's computer.
        let mut script = self.script.clone();

        // Set the environment variables
        for (key, value) in context {
            script = script.env(format!("GW_{key}"), value);
        }

        // Start the shell script
        info!(
            "Running script {:?} {}in {}.",
            self.command,
            if self.runs_in_shell {
                "in a shell "
            } else {
                ""
            },
            self.directory,
        );
        let child = script.reader()?;

        let reader = BufReader::new(&child).lines();
        let command_id = self.command.as_str();
        for line in reader {
            match line {
                Ok(line) => debug!("[{command_id}] {line}"),
                Err(_) => debug!("[{command_id}] <output cannot be parsed>"),
            }
        }

        if let Ok(Some(output)) = child.try_wait() {
            if output.status.success() {
                info!("Script {:?} finished successfully.", self.command);
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
        Ok(self.run_inner(context)?)
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

    const ECHO_TEST: &str = "echo test";
    const EXIT_NONZERO: &str = "exit 1";

    #[cfg(unix)]
    const ECHO_INVALID_UNICODE: &str =
        "python -c \"import sys; sys.stdout.buffer.write(b'\\xc3\\x28')\"; sys.stdout.flush()";
    #[cfg(unix)]
    const ECHO_STDERR: &str = "echo err >&2";
    #[cfg(unix)]
    const PRINTENV: &str = "printenv";

    #[cfg(not(unix))]
    const PRINTENV: &str = "set";

    #[test]
    fn it_should_create_new_script() {
        let command = String::from(ECHO_TEST);
        let action = ScriptAction::new(String::from("."), command, true).unwrap();

        assert_eq!("echo", action.command);
        assert_eq!(".", action.directory);
    }

    #[test]
    fn it_should_fail_if_command_is_invalid() {
        let result = ScriptAction::new(String::from("."), String::from("echo 'test"), false);

        assert!(
            matches!(result, Err(ScriptError::CommandParseFailure(_))),
            "{result:?} should match CommandParseFailure"
        );
    }

    #[test]
    fn it_should_run_the_script() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = String::from(ECHO_TEST);
        let action = ScriptAction::new(String::from("."), command, true)?;

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output("echo", |lines| {
            assert_eq!(vec!["test"], lines);
        });

        Ok(())
    }

    #[test]
    fn it_should_set_the_env_vars() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = String::from(PRINTENV);
        let action = ScriptAction::new(String::from("."), command, true)?;

        let context: Context = HashMap::from([
            ("TRIGGER_NAME", "TEST-TRIGGER".to_string()),
            ("CHECK_NAME", "TEST-CHECK".to_string()),
        ]);
        action.run_inner(&context)?;

        validate_output(PRINTENV, |lines| {
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

        let command = String::from(PRINTENV);
        let action = ScriptAction::new(String::from("."), command, true)?;

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output(PRINTENV, |lines| {
            assert!(lines.contains(&"GW_TEST=GW_TEST"));
        });

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn it_should_catch_error_output() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = String::from(ECHO_STDERR);
        let action = ScriptAction::new(String::from("."), command, true)?;

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output("echo", |lines| {
            assert_eq!(vec!["err"], lines);
        });

        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn it_should_record_if_the_script_returns_non_utf8() -> Result<(), ScriptError> {
        testing_logger::setup();

        let command = String::from(ECHO_INVALID_UNICODE);
        let action = ScriptAction::new(String::from("."), command, false)?;

        let context: Context = HashMap::new();
        action.run_inner(&context)?;

        validate_output("python", |lines| {
            assert_eq!(vec!["<output cannot be parsed>"], lines);
        });

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_fails() -> Result<(), ScriptError> {
        let command = String::from(EXIT_NONZERO);
        let action = ScriptAction::new(String::from("."), command, true)?;

        let context: Context = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::NonZeroExitcode(1))),
            "{result:?} should match non zero exit code"
        );

        Ok(())
    }
}
