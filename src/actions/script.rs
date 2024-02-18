use super::{Action, ActionError};
use duct_sh::sh_dangerous;
use log::{debug, error};
use std::collections::HashMap;
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

    fn run_inner(&self, context: &HashMap<String, String>) -> Result<String, ScriptError> {
        let mut env: HashMap<String, String> = HashMap::from([
            ("CI".to_string(), "true".to_string()),
            ("GW_ACTION_NAME".to_string(), ACTION_NAME.to_string()),
            ("GW_DIRECTORY".to_string(), self.directory.clone()),
        ]);
        env.extend(
            context
                .iter()
                .map(|(k, v)| (format!("GW_{k}"), v.to_owned())),
        );

        // We can run `sh_dangerous`, because it is on the user's computer.
        let output = sh_dangerous(&self.command)
            .full_env(env)
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
    fn run(&self, context: &HashMap<String, String>) -> Result<(), ActionError> {
        debug!(
            "Running script: {} in directory {}.",
            self.command, self.directory
        );

        match self.run_inner(context) {
            Ok(result) => {
                debug!("Command success, output:");
                result.lines().for_each(|line| {
                    debug!("{line}");
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

    #[test]
    fn it_should_create_new_script() {
        let action = ScriptAction::new(String::from("."), String::from("echo test"));

        assert_eq!("echo test", action.command);
        assert_eq!(".", action.directory);
    }

    #[test]
    fn it_should_run_the_script() -> Result<(), ScriptError> {
        let action = ScriptAction::new(String::from("."), String::from("echo test"));

        let context: HashMap<String, String> = HashMap::new();
        let output = action.run_inner(&context)?;
        assert_eq!("test", output);

        Ok(())
    }

    #[test]
    fn it_should_set_the_env_vars() -> Result<(), ScriptError> {
        let action = ScriptAction::new(String::from("."), String::from("printenv"));

        let context: HashMap<String, String> = HashMap::from([
            ("TRIGGER_NAME".to_string(), "TEST-TRIGGER".to_string()),
            ("CHECK_NAME".to_string(), "TEST-CHECK".to_string()),
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
    fn it_should_catch_error_output() -> Result<(), ScriptError> {
        let action = ScriptAction::new(String::from("."), String::from("echo err >&2"));

        let context: HashMap<String, String> = HashMap::new();
        let output = action.run_inner(&context)?;
        assert_eq!("err", output);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_fails() -> Result<(), ScriptError> {
        let action = ScriptAction::new(String::from("."), String::from("false"));

        let context: HashMap<String, String> = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::NonZeroExitcode(1, _))),
            "{result:?} should match non zero exit code"
        );

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_returns_non_utf8() -> Result<(), ScriptError> {
        let action =
            ScriptAction::new(String::from("."), String::from("/bin/echo -e '\\xc3\\x28'"));

        let context: HashMap<String, String> = HashMap::new();
        let result = action.run_inner(&context);
        assert!(
            matches!(result, Err(ScriptError::NonUtf8Return)),
            "{result:?} should match non utf8 return"
        );

        Ok(())
    }
}
