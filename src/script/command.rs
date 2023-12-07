use crate::repository::{git::GitRepository, Repository};
use std::process::{Child, Command};

pub fn run_command(repo: &GitRepository, command: &str) -> Result<Child, String> {
    let args = shlex::split(command).ok_or(String::from(""))?;

    let child = Command::new(&args[0])
        .args(&args[1..])
        .current_dir(repo.get_directory())
        .env_clear()
        .envs(repo.get_envs())
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(child)
}
