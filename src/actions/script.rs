use super::{Action, ActionError};
use crate::context::Context;
use duct_sh::sh_dangerous;
use log::{debug, error, info};
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
}

/// Custom error describing the error cases for the ScriptAction.
#[derive(Debug, Error)]
pub enum ScriptError {
    /// The underlying Rust command creation failed. The parameter contains the error.
    #[error("the script cannot run: {0}")]
    ScriptFailure(#[from] std::io::Error),
    /// The script returned a non-zero exit code, usually meaning it failed to start
    /// or encountered an error. The parameters are the exit code and the failed output.
    #[error("the script returned non-zero exit code {0} with message: {1}")]
    NonZeroExitcode(i32, String),
    /// The script output contains non-UTF8 characters.
    #[error("the script returned invalid characters")]
    NonUtf8Return,
}

impl From<ScriptError> for ActionError {
    fn from(value: ScriptError) -> Self {
        match value {
            ScriptError::ScriptFailure(_)
            | ScriptError::NonZeroExitcode(_, _)
            | ScriptError::NonUtf8Return => ActionError::FailedAction(value.to_string()),
        }
    }
}

impl ScriptAction {
    /// Creates a new script to be started in the given directory.
    pub fn new(directory: String, command: String) -> Self {
        ScriptAction { directory, command }
    }

    fn run_inner(&self, context: &Context) -> Result<String, ScriptError> {
        // We can run `sh_dangerous`, because it is on the user's computer.
        let mut command = sh_dangerous(&self.command);

        // Set the environment variables
        command = command.env("CI", "true");
        command = command.env("GW_ACTION_NAME", ACTION_NAME);
        command = command.env("GW_DIRECTORY", &self.directory);
        for (key, value) in context {
            command = command.env(format!("GW_{key}"), value);
        }

        // Start the shell script
        let output = command
            .stderr_to_stdout()
            .stdout_capture()
            .dir(&self.directory)
            .unchecked()
            .run()?;

        let output_str =
            std::str::from_utf8(&output.stdout).map_err(|_| ScriptError::NonUtf8Return)?;
        let output_str = output_str.trim_end().to_string();

        if output.status.success() {
            Ok(output_str)
        } else {
            Err(ScriptError::NonZeroExitcode(
                output.status.code().unwrap_or(-1),
                output_str,
            ))
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
            Ok(result) => {
                info!("Script {:?} finished successfully.", self.command);
                result.lines().for_each(|line| {
                    debug!("[{}]  {line}", self.command);
                });
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

    const ECHO_TEST: &str = "printf test";
    const ECHO_STDERR: &str = "printf err >&2";
    const ECHO_INVALID_UNICODE: &str = "/bin/printf '\\xc3\\x28'";
    const PRINTENV: &str = "printenv";
    const FALSE: &str = "false";

    #[test]
    fn it_should_create_new_script() {
        let command = String::from(ECHO_TEST);
        let action = ScriptAction::new(String::from("."), command);

        assert_eq!(ECHO_TEST, action.command);
        assert_eq!(".", action.directory);
    }

    #[test]
    fn it_should_run_the_script() -> Result<(), ScriptError> {
        let command = String::from(ECHO_TEST);
        let action = ScriptAction::new(String::from("."), command);

        let context: Context = HashMap::new();
        let output = action.run_inner(&context)?;
        assert_eq!("test", output);

        Ok(())
    }

    #[test]
    fn it_should_set_the_env_vars() -> Result<(), ScriptError> {
        let command = String::from(PRINTENV);
        let action = ScriptAction::new(String::from("."), command);

        let context: Context = HashMap::from([
            ("TRIGGER_NAME", "TEST-TRIGGER".to_string()),
            ("CHECK_NAME", "TEST-CHECK".to_string()),
        ]);
        let output = action.run_inner(&context)?;
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.contains(&"CI=true"));
        assert!(lines.contains(&"GW_TRIGGER_NAME=TEST-TRIGGER"));
        assert!(lines.contains(&"GW_CHECK_NAME=TEST-CHECK"));
        assert!(lines.contains(&"GW_ACTION_NAME=SCRIPT"));
        assert!(lines.contains(&"GW_DIRECTORY=."));

        Ok(())
    }

    #[test]
    fn it_should_keep_the_already_set_env_vars() -> Result<(), ScriptError> {
        std::env::set_var("GW_TEST", "GW_TEST");

        let command = String::from(PRINTENV);
        let action = ScriptAction::new(String::from("."), command);

        let context: Context = HashMap::new();
        let output = action.run_inner(&context)?;
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.contains(&"GW_TEST=GW_TEST"));

        Ok(())
    }

    #[test]
    fn it_should_catch_error_output() -> Result<(), ScriptError> {
        let command = String::from(ECHO_STDERR);
        let action = ScriptAction::new(String::from("."), command);

        let context: Context = HashMap::new();
        let output = action.run_inner(&context)?;
        assert_eq!("err", output);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_fails() -> Result<(), ScriptError> {
        let command = String::from(FALSE);
        let action = ScriptAction::new(String::from("."), command);

        let context: Context = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::NonZeroExitcode(1, _))),
            "{result:?} should match non zero exit code"
        );

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_returns_non_utf8() -> Result<(), ScriptError> {
        let command = String::from(ECHO_INVALID_UNICODE);
        let action = ScriptAction::new(String::from("."), command);

        let context: Context = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::NonUtf8Return)),
            "{result:?} should match non utf8 return"
        );

        Ok(())
    }
}
