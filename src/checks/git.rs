use self::repository::{shorthash, GitRepository, GitRepositoryInformation};
use super::{Check, CheckError};
use crate::context::Context;
use std::fmt::Debug;
use thiserror::Error;

mod config;
mod credentials;
mod known_hosts;
mod repository;

use config::setup_gitconfig;
pub use credentials::CredentialAuth;
use known_hosts::setup_known_hosts;

const CHECK_NAME: &str = "GIT";

/// A check to fetch and pull a local git repository.
///
/// This will update the repository, if there are any changes.
/// In case there are any local changes, these might be erased.
pub struct GitCheck(pub GitRepository);

/// A custom error describing the error cases for the GitCheck.
#[derive(Debug, Error)]
pub enum GitError {
    /// The directory is not a valid git repository.
    #[error("{0} is not a valid git repository ({1})")]
    NotAGitRepository(String, String),
    /// Cannot parse HEAD, either stuck an unborn branch or some deleted reference
    #[error("HEAD is invalid, probably points to invalid commit")]
    NoHead,
    /// There is no branch in the repository currently. It can be a repository
    /// without any branch, or checked out on a commit.
    #[error("repository is not on a branch, checkout or create a commit first")]
    NotOnABranch,
    /// There is no remote for the current branch. This usually because the branch hasn't been pulled.
    #[error("branch {0} doesn't have a remote, push your commits first")]
    NoRemoteForBranch(String),
    /// There are changes in the directory, avoiding pulling. This is a safety mechanism to avoid pulling
    /// over local changes, to not overwrite anything important.
    #[error("there are uncommited changes in the directory")]
    DirtyWorkingTree,
    /// Cannot load the git config
    #[error("cannot load git config")]
    ConfigLoadingFailed,
    /// Cannot create the ssh config
    #[error("cannot create ssh config")]
    SshConfigFailed,
    /// Cannot fetch the current branch. This can be a network failure, authentication error or many other things.
    #[error("cannot fetch ({0})")]
    FetchFailed(String),
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
            GitError::NotAGitRepository(_, _)
            | GitError::NoHead
            | GitError::NotOnABranch
            | GitError::NoRemoteForBranch(_) => CheckError::Misconfigured(value.to_string()),
            GitError::ConfigLoadingFailed | GitError::SshConfigFailed => {
                CheckError::PermissionDenied(value.to_string())
            }
            GitError::DirtyWorkingTree | GitError::MergeConflict => {
                CheckError::Conflict(value.to_string())
            }
            GitError::FetchFailed(_) | GitError::FailedSettingHead(_) => {
                CheckError::FailedUpdate(value.to_string())
            }
        }
    }
}

impl GitCheck {
    /// Open the git repository at the given directory.
    pub fn open_inner(directory: &str) -> Result<Self, CheckError> {
        let repo = GitRepository::open(directory)?;
        Ok(GitCheck(repo))
    }

    pub fn open(directory: &str, additional_host: Option<String>) -> Result<Self, CheckError> {
        setup_known_hosts(additional_host)?;
        setup_gitconfig(directory)?;

        let repo = GitRepository::open(directory)?;
        Ok(GitCheck(repo))
    }

    pub fn set_auth(&mut self, auth: CredentialAuth) {
        self.0.set_auth(auth);
    }

    fn check_inner(&mut self, context: &mut Context) -> Result<bool, GitError> {
        let GitCheck(repo) = self;

        // Load context data from repository information
        let information = repo.get_repository_information()?;
        context.insert("CHECK_NAME", CHECK_NAME.to_string());
        match information {
            GitRepositoryInformation::Branch {
                ref_type,
                ref_name,
                branch_name,
                commit_sha,
                commit_short_sha,
                remote_name,
                remote_url,
            } => {
                context.insert("GIT_REF_TYPE", ref_type);
                context.insert("GIT_REF_NAME", ref_name);
                context.insert("GIT_BRANCH_NAME", branch_name);
                context.insert("GIT_BEFORE_COMMIT_SHA", commit_sha.clone());
                context.insert("GIT_BEFORE_COMMIT_SHORT_SHA", commit_short_sha.clone());
                context.insert("GIT_COMMIT_SHA", commit_sha);
                context.insert("GIT_COMMIT_SHORT_SHA", commit_short_sha);
                context.insert("GIT_REMOTE_NAME", remote_name);
                context.insert("GIT_REMOTE_URL", remote_url);
            }
            GitRepositoryInformation::Reference {
                ref_type,
                ref_name,
                commit_sha,
                commit_short_sha,
            } => {
                context.insert("GIT_REF_TYPE", ref_type);
                context.insert("GIT_REF_NAME", ref_name);
                context.insert("GIT_BEFORE_COMMIT_SHA", commit_sha);
                context.insert("GIT_BEFORE_COMMIT_SHORT_SHA", commit_short_sha);
            }
        }

        // Pull repository contents and report
        let fetch_commit = repo.fetch()?;
        if repo.check_if_updatable(&fetch_commit)? && repo.pull(&fetch_commit)? {
            context.insert("GIT_COMMIT_SHA", fetch_commit.id().to_string());
            context.insert(
                "GIT_COMMIT_SHORT_SHA",
                shorthash(&fetch_commit.id().to_string()),
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Check for GitCheck {
    /// Fetch and pull changes from the remote repository on the current branch.
    /// It returns true if the pull was successful and there are new changes.
    fn check(&mut self, context: &mut Context) -> Result<bool, CheckError> {
        let update_successful = self.check_inner(context)?;

        Ok(update_successful)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duct::cmd;
    use rand::distributions::{Alphanumeric, DistString};
    use std::{collections::HashMap, error::Error, fs, path::Path};

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

    fn create_tag(path: &str, tag: &str) -> Result<(), Box<dyn Error>> {
        cmd!("git", "tag", tag).dir(path).read()?;
        cmd!("git", "push", "--tags").dir(path).read()?;

        Ok(())
    }

    fn get_tags(path: &str) -> Result<String, Box<dyn Error>> {
        let tags = cmd!("git", "tag", "-l").dir(path).read()?;

        Ok(tags)
    }

    fn get_last_commit(path: &str) -> Result<String, Box<dyn Error>> {
        let commit_sha = cmd!("git", "rev-parse", "HEAD").dir(path).read()?;

        Ok(commit_sha)
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

        let _ = GitCheck::open_inner(&local)?;

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_path_is_invalid() -> Result<(), Box<dyn Error>> {
        let error = GitCheck::open_inner("/path/to/nowhere").err().unwrap();

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

        let failing_check = GitCheck::open_inner(&local);
        let error = failing_check.err().unwrap();

        assert!(
            matches!(error, CheckError::Misconfigured(_)),
            "{error:?} should be Misconfigured"
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

        let failing_check = GitCheck::open_inner(&local);
        let error = failing_check.err().unwrap();

        assert!(
            matches!(error, CheckError::Misconfigured(_)),
            "{error:?} should be Misconfigured"
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_the_remote_didnt_change() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let mut check = GitCheck::open_inner(&local)?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context)?;
        assert!(!is_pulled);

        // It should set the context keys
        let commit_sha = get_last_commit(&local)?;
        let remote_path = fs::canonicalize(format!("{local}-remote"))?;
        let remote = remote_path.to_str().unwrap();
        assert_eq!("branch", context.get("GIT_REF_TYPE").unwrap());
        assert_eq!("refs/heads/master", context.get("GIT_REF_NAME").unwrap());
        assert_eq!("master", context.get("GIT_BRANCH_NAME").unwrap());
        assert_eq!(&commit_sha, context.get("GIT_BEFORE_COMMIT_SHA").unwrap());
        assert_eq!(
            &commit_sha[0..7],
            context.get("GIT_BEFORE_COMMIT_SHORT_SHA").unwrap()
        );
        assert_eq!("origin", context.get("GIT_REMOTE_NAME").unwrap());
        assert_eq!(
            remote,
            fs::canonicalize(context.get("GIT_REMOTE_URL").unwrap())?
                .to_str()
                .unwrap()
        );

        cleanup_repository(&local)?;

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_remote_changes() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local).unwrap(); // ?;

        // Create another repository and push a new commit
        create_other_repository(&local).unwrap(); // ?;

        let before_commit_sha = get_last_commit(&local).unwrap(); // ?;
        let mut check = GitCheck::open_inner(&local).unwrap(); // ?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context).unwrap(); // ?;
        assert!(is_pulled);

        // The pushed file should be pulled
        assert!(Path::new(&format!("{local}/2")).exists());

        // It should set the context keys
        let remote_path = fs::canonicalize(format!("{local}-remote")).unwrap(); // ?;
        let remote = remote_path.to_str().unwrap();
        let commit_sha = get_last_commit(&local).unwrap(); // ?;
        assert_eq!("branch", context.get("GIT_REF_TYPE").unwrap());
        assert_eq!("refs/heads/master", context.get("GIT_REF_NAME").unwrap());
        assert_eq!("master", context.get("GIT_BRANCH_NAME").unwrap());
        assert_eq!(
            &before_commit_sha,
            context.get("GIT_BEFORE_COMMIT_SHA").unwrap()
        );
        assert_eq!(
            &before_commit_sha[0..7],
            context.get("GIT_BEFORE_COMMIT_SHORT_SHA").unwrap()
        );
        assert_eq!(&commit_sha, context.get("GIT_COMMIT_SHA").unwrap());
        assert_eq!(
            &commit_sha[0..7],
            context.get("GIT_COMMIT_SHORT_SHA").unwrap()
        );
        assert_eq!("origin", context.get("GIT_REMOTE_NAME").unwrap());
        assert_eq!(
            remote,
            fs::canonicalize(context.get("GIT_REMOTE_URL").unwrap())?
                .to_str()
                .unwrap()
        );

        cleanup_repository(&local).unwrap(); // ?;

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_remote_changes_even_with_tags() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local).unwrap(); // ?;

        // Create another repository, push a new commit and add a tag
        create_other_repository(&local).unwrap(); // ?;
        create_tag(&format!("{local}-other"), "v0.1.0").unwrap(); // ?;

        let mut check = GitCheck::open_inner(&local).unwrap(); // ?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context).unwrap(); // ?;
        assert!(is_pulled);

        // The pushed file should be pulled
        assert!(Path::new(&format!("{local}/2")).exists());

        // Tag should be downloaded
        let tags = get_tags(&local).unwrap(); // ?;
        assert_eq!(tags, "v0.1.0");

        cleanup_repository(&local).unwrap(); // ?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_working_tree_is_dirty() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local).unwrap(); // ?;

        // Create another repository and push a new commit
        create_other_repository(&local).unwrap(); // ?;

        // Add uncommited modification to emulate a dirty working tree
        fs::write(format!("{local}/1"), "22").unwrap(); // ?;

        let mut check = GitCheck::open_inner(&local).unwrap(); // ?;
        let mut context: Context = HashMap::new();
        let error = check.check_inner(&mut context).err().unwrap();

        assert!(
            matches!(error, GitError::DirtyWorkingTree),
            "{error:?} should be DirtyWorkingTree"
        );

        // The pushed file should be pulled
        assert!(!Path::new(&format!("{local}/2")).exists());

        cleanup_repository(&local).unwrap(); // ?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_there_is_a_merge_conflict() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local).unwrap(); // ?;

        // Create another repository and push a new commit
        create_other_repository(&local).unwrap(); // ?;

        // Modify the same file in both directories to create a merge conflict
        create_merge_conflict(&local).unwrap(); // ?;

        let mut check = GitCheck::open_inner(&local).unwrap(); // ?;
        let mut context: Context = HashMap::new();
        let error = check.check_inner(&mut context).err().unwrap();

        assert!(
            matches!(error, GitError::MergeConflict),
            "{error:?} should be MergeConflict"
        );

        cleanup_repository(&local).unwrap(); // ?;

        Ok(())
    }

    #[test]
    fn it_should_fail_if_repository_is_not_accessible() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local).unwrap(); // ?;

        // Create another repository and push a new commit
        create_other_repository(&local).unwrap(); // ?;

        // Set repository to readonly
        let mut perms = fs::metadata(&local)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&local, perms).unwrap(); // ?;

        let mut check: GitCheck = GitCheck::open_inner(&local).unwrap(); // ?;
        let mut context: Context = HashMap::new();
        let error = check.check_inner(&mut context).err().unwrap();

        assert!(
            matches!(error, GitError::FailedSettingHead(_)),
            "{error:?} should be FailedSettingHead"
        );

        // Set repository to readonly
        let mut perms = fs::metadata(&local)?.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs::set_permissions(&local, perms).unwrap(); // ?;

        cleanup_repository(&local).unwrap(); // ?;

        Ok(())
    }
}
