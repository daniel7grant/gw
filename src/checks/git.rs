use self::repository::GitRepository;

use super::Check;
use crate::Result;

mod repository;
mod credentials;

/// A check to fetch and pull a local git repository.
pub struct GitCheck(pub GitRepository);

impl GitCheck {
    /// Open the git repository at the given directory.
    pub fn open(directory: &str) -> Result<Self> {
        let repo = GitRepository::open(directory)?;
        Ok(GitCheck(repo))
    }
}

impl Check for GitCheck {
    /// Fetch and pull changes from the remote repository on the current branch.
    /// It returns true if the pull was successful and there are new changes.
    fn check(&mut self) -> Result<bool> {
        let GitCheck(repo) = self;
        let fetch_commit = repo.fetch()?;
        if repo.check_if_updatable(&fetch_commit)? && repo.pull(&fetch_commit)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use duct::cmd;
    use rand::distributions::{Alphanumeric, DistString};
    use std::{fs, path::Path};

    fn get_random_id() -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
    }

    fn create_empty_repository(local: &str) -> Result<()> {
        let remote = format!("{local}-remote");

        // Create directory and repository in it
        fs::create_dir(&remote)?;
        cmd!("git", "init", "--bare").dir(&remote).read()?;
        cmd!("git", "clone", &remote, &local).read()?;
        fs::write(&format!("{local}/1"), "1")?;
        cmd!("git", "add", "-A").dir(&local).read()?;
        cmd!("git", "commit", "-m1").dir(&local).read()?;
        cmd!("git", "push", "origin", "master").dir(&local).read()?;

        Ok(())
    }

    fn create_other_repository(local: &str) -> Result<()> {
        let remote = format!("{local}-remote");
        let other = format!("{local}-other");

        // Create another directory to push the changes
        cmd!("git", "clone", &remote, &other).read()?;
        fs::write(&format!("{other}/2"), "2")?;
        cmd!("git", "add", "-A").dir(&other).read()?;
        cmd!("git", "commit", "-m1").dir(&other).read()?;
        cmd!("git", "push", "origin", "master").dir(&other).read()?;

        Ok(())
    }

    fn cleanup_repository(local: &str) -> Result<()> {
        let remote = format!("{local}-remote");
        let other = format!("{local}-other");

        fs::remove_dir_all(local)?;
        fs::remove_dir_all(remote)?;
        if Path::new(&other).exists() {
            fs::remove_dir_all(other)?;
        }

        Ok(())
    }

    #[test]
    fn it_should_open_a_repository() -> Result<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let _ = GitCheck::open(&local)?;

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_the_remote_didnt_change() -> Result<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let mut check = GitCheck::open(&local)?;
        let is_pulled = check.check()?;
        assert!(!is_pulled);

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_remote_changes() -> Result<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit 
        create_other_repository(&local)?;

        let mut check = GitCheck::open(&local)?;
        let is_pulled = check.check()?;
        assert!(is_pulled);

        // The pushed file should be pulled
        assert!(Path::new(&format!("{local}/2")).exists());

        cleanup_repository(&local)?;

        Ok(())
    }
}
