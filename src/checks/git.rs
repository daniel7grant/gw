use self::repository::GitRepository;
use super::{Check, CheckError};
use std::fmt::Debug;
use thiserror::Error;

mod credentials;
mod repository;

/// A check to fetch and pull a local git repository.
///
/// This will update the repository, if there are any changes.
/// In case there are any local changes, these might be erased.
pub struct GitCheck(pub GitRepository);

/// A custom error describing the error cases for the GitCheck.
#[derive(Debug, Error)]
pub enum GitError {
    /// The directory is not a valid git repository.
    #[error("{0} does not exist or not a git repository")]
    NotAGitRepository(String),
    /// There is no branch in the repository currently. It can be a repository
    /// without any branch, or checked out on a commit.
    #[error("repository is not on a branch")]
    NotOnABranch,
    /// There is no remote for the current branch. This usually because the branch hasn't been pulled.
    #[error("branch {0} doesn't have a remote")]
    NoRemoteForBranch(String),
    /// There are changes in the directory, avoiding pulling. This is a safety mechanism to avoid pulling
    /// over local changes, to not overwrite anything important.
    #[error("there are uncommited changes in the directory")]
    DirtyWorkingTree,
    /// Cannot load the git config
    #[error("cannot load git config")]
    ConfigLoadingFailed,
    /// Cannot fetch the current branch. This is possibly a network failure.
    #[error("cannot fetch, might be a network error")]
    FetchFailed,
    /// Cannot pull updates to the current branch. This means either the merge analysis failed
    /// or there is a merge conflict.
    #[error("cannot update branch, this is likely a merge conflict")]
    MergeConflict,
    /// Cannot set the HEAD to the fetch commit.
    #[error("could not set HEAD to fetch commit {0}")]
    FailedSettingHead(String),
}

impl From<GitError> for CheckError {
    fn from(value: GitError) -> Self {
        match value {
            GitError::NotAGitRepository(_)
            | GitError::NotOnABranch
            | GitError::NoRemoteForBranch(_) => CheckError::Misconfigured(value.to_string()),
            GitError::ConfigLoadingFailed => CheckError::PermissionDenied(value.to_string()),
            GitError::DirtyWorkingTree | GitError::MergeConflict => {
                CheckError::Conflict(value.to_string())
            }
            GitError::FetchFailed | GitError::FailedSettingHead(_) => {
                CheckError::FailedUpdate(value.to_string())
            }
        }
    }
}

impl GitCheck {
    /// Open the git repository at the given directory.
    pub fn open(directory: &str) -> Result<Self, CheckError> {
        let repo = GitRepository::open(directory)?;
        Ok(GitCheck(repo))
    }

    fn check_inner(&mut self) -> Result<bool, GitError> {
        let GitCheck(repo) = self;
        let fetch_commit = repo.fetch()?;
        if repo.check_if_updatable(&fetch_commit)? && repo.pull(&fetch_commit)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Check for GitCheck {
    /// Fetch and pull changes from the remote repository on the current branch.
    /// It returns true if the pull was successful and there are new changes.
    fn check(&mut self) -> Result<bool, CheckError> {
        let update_successful = self.check_inner()?;

        Ok(update_successful)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duct::cmd;
    use rand::distributions::{Alphanumeric, DistString};
    use std::{error::Error, fs, path::Path};

    fn get_random_id() -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
    }

    fn create_empty_repository(local: &str) -> Result<(), Box<dyn Error>> {
        let remote = format!("{local}-remote");

        // Create directory and repository in it
        fs::create_dir(&remote)?;
        cmd!("git", "init", "--bare").dir(&remote).read()?;
        cmd!("git", "clone", &remote, &local).read()?;
        fs::write(format!("{local}/1"), "1")?;
        cmd!("git", "add", "-A").dir(local).read()?;
        cmd!("git", "commit", "-m1").dir(local).read()?;
        cmd!("git", "push", "origin", "master").dir(local).read()?;

        Ok(())
    }

    fn create_other_repository(local: &str) -> Result<(), Box<dyn Error>> {
        let remote = format!("{local}-remote");
        let other = format!("{local}-other");

        // Create another directory to push the changes
        cmd!("git", "clone", &remote, &other).read()?;
        fs::write(format!("{other}/2"), "2")?;
        cmd!("git", "add", "-A").dir(&other).read()?;
        cmd!("git", "commit", "-m1").dir(&other).read()?;
        cmd!("git", "push", "origin", "master").dir(other).read()?;

        Ok(())
    }

    fn cleanup_repository(local: &str) -> Result<(), Box<dyn Error>> {
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

    fn create_failing_repository(local: &str, create_commit: bool) -> Result<(), Box<dyn Error>> {
        fs::create_dir(local)?;
        cmd!("git", "init").dir(local).read()?;

        if create_commit {
            fs::write(format!("{local}/1"), "1")?;
            cmd!("git", "add", "-A").dir(local).read()?;
            cmd!("git", "commit", "-m1").dir(local).read()?;
        }

        Ok(())
    }

    fn create_merge_conflict(local: &str) -> Result<(), Box<dyn Error>> {
        let other = format!("{local}-other");

        fs::write(format!("{local}/1"), "11")?;
        cmd!("git", "add", "-A").dir(local).read()?;
        cmd!("git", "commit", "-m1").dir(local).read()?;

        fs::write(format!("{other}/1"), "12")?;
        cmd!("git", "add", "-A").dir(&other).read()?;
        cmd!("git", "commit", "-m2").dir(other).read()?;

        Ok(())
    }

    #[test]
    fn it_should_open_a_repository() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let _ = GitCheck::open(&local)?;

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_path_is_invalid() -> Result<(), Box<dyn Error>> {
        let error = GitCheck::open("/path/to/nowhere").err().unwrap();

        assert!(
            matches!(error, CheckError::Misconfigured(_)),
            "{error:?} should be Misconfigured"
        );

        Ok(())
    }

    #[test]
    fn it_should_fail_if_we_are_not_on_a_branch() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        // Don't create commit to create an empty repository
        create_failing_repository(&local, false)?;

        let mut check: GitCheck = GitCheck::open(&local)?;
        let error = check.check_inner().err().unwrap();

        assert!(
            matches!(error, GitError::NotOnABranch),
            "{error:?} should be NotOnABranch"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_there_is_no_remote() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        // Don't create commit to create an empty repository
        create_failing_repository(&local, true)?;

        let mut check: GitCheck = GitCheck::open(&local)?;
        let error = check.check_inner().err().unwrap();

        assert!(
            matches!(error, GitError::NoRemoteForBranch(_)),
            "{error:?} should be NoRemoteForBranch"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_the_remote_didnt_change() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let mut check = GitCheck::open(&local)?;
        let is_pulled = check.check_inner()?;
        assert!(!is_pulled);

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_remote_changes() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit
        create_other_repository(&local)?;

        let mut check = GitCheck::open(&local)?;
        let is_pulled = check.check_inner()?;
        assert!(is_pulled);

        // The pushed file should be pulled
        assert!(Path::new(&format!("{local}/2")).exists());

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_there_is_a_merge_conflict() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit
        create_other_repository(&local)?;

        // Modify the same file in both directories to create a merge conflict
        create_merge_conflict(&local)?;

        let mut check = GitCheck::open(&local)?;
        let error = check.check_inner().err().unwrap();

        assert!(
            matches!(error, GitError::MergeConflict),
            "{error:?} should be MergeConflict"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_repository_is_not_accessible() -> Result<(), Box<dyn Error>> {
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
        let error = check.check_inner().err().unwrap();

        assert!(
            matches!(error, GitError::FailedSettingHead(_)),
            "{error:?} should be FailedSettingHead"
        );

        // Set repository to readonly
        let mut perms = fs::metadata(&local)?.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs::set_permissions(&local, perms)?;

        cleanup_repository(&local)?;

        Ok(())
    }
}
