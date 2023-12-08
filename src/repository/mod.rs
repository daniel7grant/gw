use std::collections::HashMap;

use self::git::GitRepository;

pub mod git;
pub mod git_credentials;

pub trait Repository
where
    Self: Sized,
{
    fn open(s: &str) -> Result<Self, String>;
    fn get_directory(self: &Self) -> String;
    fn get_envs(self: &Self) -> HashMap<String, String>;
    fn check_for_updates(self: &Self) -> Result<bool, String>;
    fn pull_updates(self: &mut Self) -> Result<bool, String>;
}

pub fn open_repository(s: &str) -> Result<GitRepository, String> {
    GitRepository::open(s)
}
