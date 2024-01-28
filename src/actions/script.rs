use duct_sh::sh_dangerous;

use super::Action;
use crate::Result;

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

impl ScriptAction {
    /// Creates a new script to be started in the given directory.
    pub fn new(directory: String, command: String) -> Self {
        ScriptAction { directory, command }
    }

    fn run_inner(&self) -> Result<String> {
        // We can run `sh_dangerous`, because it is one the user's computer.
        let output = sh_dangerous(&self.command)
            .stderr_to_stdout()
            .dir(&self.directory)
            .read()?;
        Ok(output)
    }
}

impl Action for ScriptAction {
    /// Run the script in a subshell (`/bin/sh` on *nix, `cmd.exe` on Windows).
    /// If the script fails, this function will result in an error.
    fn run(&self) -> Result<()> {
        println!(
            "Running script: {} in directory {}.",
            self.command, self.directory
        );

        match self.run_inner() {
            Ok(result) => println!("Command success: {result}"),
            Err(err) => println!("Failed: {err}"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;

    #[test]
    fn it_should_create_new_script() {
        let action = ScriptAction::new(String::from("."), String::from("echo test"));

        assert_eq!("echo test", action.command);
        assert_eq!(".", action.directory);
    }

    #[test]
    fn it_should_run_the_script() -> Result<()> {
        let action = ScriptAction::new(String::from("."), String::from("echo test"));

        let output = action.run_inner()?;
        assert_eq!("test", output);

        Ok(())
    }

    #[test]
    fn it_should_catch_error_output() -> Result<()> {
        let action = ScriptAction::new(String::from("."), String::from("echo err >&2"));

        let output = action.run_inner()?;
        assert_eq!("err", output);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_script_errors() -> Result<()> {
        let action = ScriptAction::new(String::from("."), String::from("false"));

        let result = action.run_inner();
        assert!(result.is_err());

        Ok(())
    }
}
