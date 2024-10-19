use duct::{cmd, Expression};
use duct_sh::sh_dangerous;
use log::{trace, warn};

pub fn create_command(original_command: &str, runs_in_shell: bool) -> Option<(String, Expression)> {
    // If we are not in a shell, test if the user might want to be in one (uses variables or pipes)
    if !runs_in_shell {
        let contains_variables = original_command
            .find('$')
            .and_then(|pos| original_command.chars().nth(pos + 1))
            .map(|ch| ch.is_ascii_alphabetic() || ch == '{')
            == Some(true);

        let contains_suspicious = original_command.contains(" | ")
            || original_command.contains(" && ")
            || original_command.contains(" || ");

        if contains_variables || contains_suspicious {
            warn!("The command {original_command:?} contains a variable or other shell-specific character: you might want to run it in a shell (-S or -P).")
        }
    }

    // We have to split the command into parts to get the command id
    let split_args = shlex::split(original_command)?;
    let (command, args) = split_args.split_first()?;

    // If we are in a shell we can `sh_dangerous`, otherwise avoid bugs around shells
    let script = if runs_in_shell {
        sh_dangerous(original_command)
    } else {
        cmd(command, args)
    };

    trace!("Parsed {original_command:?} to {script:?}.");

    Some((command.clone(), script))
}
