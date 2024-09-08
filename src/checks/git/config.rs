use super::GitError;
use dirs::home_dir;
use std::{fs::File, io::Write, path::PathBuf};

/// Setup the gitconfig file.
///
/// Git will fail if we are trying to access a directory with a different user than ours.
/// To avoid this (mainly to make it work in containers), we are once again choosing usability.
/// In case there is no gitconfig file (usually in containers), we are creating it and
/// setting the current directory as safe directory inside.
pub fn setup_gitconfig(directory: &str) -> Result<(), GitError> {
    let home = home_dir().unwrap_or(PathBuf::from("~"));
    let config = home.join(".gitconfig");

    if !config.exists() {
        let mut config_file = File::create(config).map_err(|_| GitError::ConfigLoadingFailed)?;
        writeln!(config_file, "[safe]\n  directory = {directory}")
            .map_err(|_| GitError::ConfigLoadingFailed)?;
    }

    Ok(())
}
