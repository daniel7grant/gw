use self::repository::GitRepository;
use super::Check;
use crate::Result as GwResult;

mod credentials;
mod repository;

/// A check to fetch and pull a local git repository.
///
/// This will update the repository, if there are any changes.
/// In case there are any local changes, these might be erased.
pub struct GitCheck(pub GitRepository);

impl GitCheck {
    /// Open the git repository at the given directory.
    pub fn open(directory: &str) -> GwResult<Self> {
        let repo = GitRepository::open(directory)?;
        Ok(GitCheck(repo))
    }
}

impl Check for GitCheck {
    /// Fetch and pull changes from the remote repository on the current branch.
    /// It returns true if the pull was successful and there are new changes.
    fn check(&mut self) -> GwResult<bool> {
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
    use super::repository::GitError;
    use super::*;
    use duct::cmd;
    use rand::distributions::{Alphanumeric, DistString};
    use std::{fs, path::Path};

    fn get_random_id() -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
    }

    fn create_empty_repository(local: &str) -> GwResult<()> {
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

    fn create_other_repository(local: &str) -> GwResult<()> {
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

    fn cleanup_repository(local: &str) -> GwResult<()> {
        let remote = format!("{local}-remote");
        let other = format!("{local}-other");

        fs::remove_dir_all(local)?;
        if Path::new(&remote).exists() {
            fs::remove_dir_all(remote)?;
        }
        if Path::new(&other).exists() {
            fs::remove_dir_all(other)?;
        }

        Ok(())
    }

    fn create_failing_repository(local: &str, create_commit: bool) -> GwResult<()> {
        fs::create_dir(&local)?;
        cmd!("git", "init").dir(&local).read()?;

        if create_commit {
            fs::write(&format!("{local}/1"), "1")?;
            cmd!("git", "add", "-A").dir(&local).read()?;
            cmd!("git", "commit", "-m1").dir(&local).read()?;
        }

        Ok(())
    }

    fn create_merge_conflict(local: &str) -> GwResult<()> {
        let other = format!("{local}-other");

        fs::write(&format!("{local}/1"), "11")?;
        cmd!("git", "add", "-A").dir(&local).read()?;
        cmd!("git", "commit", "-m1").dir(&local).read()?;

        fs::write(&format!("{other}/1"), "12")?;
        cmd!("git", "add", "-A").dir(&other).read()?;
        cmd!("git", "commit", "-m2").dir(&other).read()?;

        Ok(())
    }

    #[test]
    fn it_should_open_a_repository() -> GwResult<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let _ = GitCheck::open(&local)?;

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_path_is_invalid() -> GwResult<()> {
        let result = GitCheck::open("/path/to/nowhere").err().unwrap();
        let error = *result.downcast::<GitError>()?;

        assert!(
            matches!(error, GitError::NotAGitRepository(_)),
            "{error:?} should be NotAGitRepository"
        );

        Ok(())
    }

    #[test]
    fn it_should_fail_if_we_are_not_on_a_branch() -> GwResult<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        // Don't create commit to create an empty repository
        create_failing_repository(&local, false)?;

        let mut check: GitCheck = GitCheck::open(&local)?;
        let result = check.check().err().unwrap();
        let error = *result.downcast::<GitError>()?;

        assert!(
            matches!(error, GitError::NotOnABranch),
            "{error:?} should be NotOnABranch"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_there_is_no_remote() -> GwResult<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        // Don't create commit to create an empty repository
        create_failing_repository(&local, true)?;

        let mut check: GitCheck = GitCheck::open(&local)?;
        let result = check.check().err().unwrap();
        let error = *result.downcast::<GitError>()?;

        assert!(
            matches!(error, GitError::NoRemoteForBranch(_)),
            "{error:?} should be NoRemoteForBranch"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_the_remote_didnt_change() -> GwResult<()> {
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
    fn it_should_return_true_if_the_remote_changes() -> GwResult<()> {
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

    #[test]
    fn it_should_fail_if_there_is_a_merge_conflict() -> GwResult<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit
        create_other_repository(&local)?;

        // Modify the same file in both directories to create a merge conflict
        create_merge_conflict(&local)?;

        let mut check = GitCheck::open(&local)?;
        let result = check.check().err().unwrap();
        let error = *result.downcast::<GitError>()?;

        assert!(
            matches!(error, GitError::MergeConflict),
            "{error:?} should be MergeConflict"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_repository_is_not_accessible() -> GwResult<()> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit
        create_other_repository(&local)?;

        // Set repository to readonly
        let mut perms = fs::metadata(&local)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&local, perms)?;

        let mut check: GitCheck = GitCheck::open(&local)?;
        let result = check.check().err().unwrap();
        let error = *result.downcast::<GitError>()?;

        assert!(
            matches!(error, GitError::FailedSettingHead(_)),
            "{error:?} should be FailedSettingHead"
        );

        // Set repository to readonly
        let mut perms = fs::metadata(&local)?.permissions();
        perms.set_readonly(false);
        fs::set_permissions(&local, perms)?;

        cleanup_repository(&local)?;

        Ok(())
    }
}
