use self::repository::GitRepository;
use super::{Check, CheckError};
use crate::context::Context;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

mod config;
mod credentials;
mod known_hosts;
mod repository;

use config::setup_gitconfig;
pub use credentials::CredentialAuth;
use known_hosts::setup_known_hosts;
use log::warn;
use repository::shorthash;

const CHECK_NAME: &str = "GIT";

#[derive(Clone, Debug)]
pub enum GitTriggerArgument {
    Push,
    Tag(String),
}

impl Display for GitTriggerArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GitTriggerArgument::Push => f.write_str("push"),
            GitTriggerArgument::Tag(pattern) => {
                if pattern == "*" {
                    f.write_str("tag")
                } else {
                    write!(f, "tag matching \"{pattern}\"")
                }
            }
        }
    }
}

/// A check to fetch and pull a local git repository.
///
/// This will update the repository, if there are any changes.
/// In case there are any local changes, these might be erased.
pub struct GitCheck {
    pub repo: GitRepository,
    pub trigger: GitTriggerArgument,
}

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
    /// Failed finding tags between the fetched commit and head. This might be a
    #[error("failed matching tags between the new commit and the branch")]
    TagMatchingFailed,
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
            GitError::FetchFailed(_)
            | GitError::FailedSettingHead(_)
            | GitError::TagMatchingFailed => CheckError::FailedUpdate(value.to_string()),
        }
    }
}

impl GitCheck {
    /// Open the git repository at the given directory.
    pub fn open_inner(directory: &str, trigger: GitTriggerArgument) -> Result<Self, CheckError> {
        let repo = GitRepository::open(directory)?;

        if let GitTriggerArgument::Tag(p) = &trigger {
            if !(p.contains('*') || p.contains('?') || p.contains('[') || p.contains('{')) {
                warn!("The tag pattern does not contain any globbing (*, ?, [] or {{}}), so it will only match \"{p}\" exactly.");
            }
        }

        Ok(GitCheck { repo, trigger })
    }

    pub fn open(
        directory: &str,
        additional_host: Option<String>,
        trigger: GitTriggerArgument,
    ) -> Result<Self, CheckError> {
        setup_known_hosts(additional_host)?;
        setup_gitconfig(directory)?;

        GitCheck::open_inner(directory, trigger)
    }

    pub fn set_auth(&mut self, auth: CredentialAuth) {
        self.repo.set_auth(auth);
    }

    fn check_inner(&mut self, context: &mut Context) -> Result<bool, GitError> {
        let GitCheck { repo, trigger } = self;

        // Load context data from repository information
        let information = repo.get_repository_information()?;
        context.insert("CHECK_NAME", CHECK_NAME.to_string());
        context.insert("GIT_BRANCH_NAME", information.branch_name);
        context.insert("GIT_BEFORE_COMMIT_SHA", information.commit_sha.to_string());
        context.insert("GIT_BEFORE_COMMIT_SHORT_SHA", information.commit_short_sha);
        context.insert("GIT_REMOTE_NAME", information.remote_name);
        context.insert("GIT_REMOTE_URL", information.remote_url);

        // Pull repository contents and report
        let fetch_commit = repo.fetch()?;
        if repo.check_if_updatable(&fetch_commit)? {
            match trigger {
                GitTriggerArgument::Push => {
                    repo.pull(fetch_commit.id())?;
                    context.insert("GIT_REF_TYPE", "branch".to_string());
                    context.insert("GIT_REF_NAME", information.ref_name);
                    context.insert("GIT_COMMIT_SHA", fetch_commit.id().to_string());
                    context.insert("GIT_COMMIT_SHORT_SHA", shorthash(&fetch_commit.id()));
                    Ok(true)
                }
                GitTriggerArgument::Tag(pattern) => {
                    let mut tags = repo.find_tags(fetch_commit.id(), pattern)?;
                    if let Some((tag_name, commit)) = tags.pop() {
                        repo.pull(commit)?;
                        context.insert("GIT_REF_TYPE", "tag".to_string());
                        context.insert("GIT_REF_NAME", format!("refs/tags/{tag_name}"));
                        context.insert("GIT_COMMIT_SHA", commit.to_string());
                        context.insert("GIT_COMMIT_SHORT_SHA", shorthash(&commit));
                        context.insert("GIT_COMMIT_TAG_NAME", tag_name.to_string());
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
            }
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
    use rand::distr::{Alphanumeric, SampleString};
    use std::{collections::HashMap, error::Error, fs, path::Path};

    fn get_random_id() -> String {
        Alphanumeric.sample_string(&mut rand::rng(), 16)
    }

    fn create_empty_repository(local: &str) -> Result<(), Box<dyn Error>> {
        let remote = format!("{local}-remote");

        // Create directory and repository in it
        fs::create_dir(&remote)?;
        cmd!("git", "init", "--bare").dir(&remote).read()?;
        cmd!("git", "clone", &remote, &local).read()?;
        create_commit(local, "1", "1")?;
        push_all(local)?;

        Ok(())
    }

    fn create_other_repository(local: &str) -> Result<(), Box<dyn Error>> {
        let remote = format!("{local}-remote");
        let other = format!("{local}-other");

        // Create another directory to push the changes
        cmd!("git", "clone", &remote, &other).read()?;
        create_commit(&other, "2", "2")?;
        push_all(&other)?;

        Ok(())
    }

    fn create_commit(path: &str, file: &str, contents: &str) -> Result<(), Box<dyn Error>> {
        fs::write(format!("{path}/{file}"), contents)?;
        cmd!("git", "add", "-A").dir(path).read()?;
        cmd!("git", "commit", "-m1").dir(path).read()?;

        Ok(())
    }

    fn push_all(path: &str) -> Result<(), Box<dyn Error>> {
        cmd!("git", "push", "origin", "master").dir(path).read()?;
        cmd!("git", "push", "--tags").dir(path).read()?;

        Ok(())
    }

    fn create_tag(path: &str, tag: &str) -> Result<(), Box<dyn Error>> {
        cmd!("git", "tag", tag).dir(path).read()?;
        push_all(path)?;

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

    fn create_failing_repository(local: &str, creating_commit: bool) -> Result<(), Box<dyn Error>> {
        fs::create_dir(local)?;
        cmd!("git", "init").dir(local).read()?;

        if creating_commit {
            create_commit(local, "1", "1")?;
        }

        Ok(())
    }

    fn create_merge_conflict(local: &str) -> Result<(), Box<dyn Error>> {
        let other = format!("{local}-other");

        create_commit(local, "1", "11")?;

        create_commit(&other, "1", "21")?;

        Ok(())
    }

    #[test]
    fn it_should_open_a_repository() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let _ = GitCheck::open_inner(&local, GitTriggerArgument::Push)?;

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_path_is_invalid() -> Result<(), Box<dyn Error>> {
        let error = GitCheck::open_inner("/path/to/nowhere", GitTriggerArgument::Push)
            .err()
            .unwrap();

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

        let failing_check = GitCheck::open_inner(&local, GitTriggerArgument::Push);
        let error = failing_check.err().unwrap();

        assert!(
            matches!(error, CheckError::Misconfigured(_)),
            "{error:?} should be Misconfigured"
        );

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_there_is_no_remote() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        // Don't create commit to create an empty repository
        create_failing_repository(&local, true)?;

        let failing_check = GitCheck::open_inner(&local, GitTriggerArgument::Push);
        let error = failing_check.err().unwrap();

        assert!(
            matches!(error, CheckError::Misconfigured(_)),
            "{error:?} should be Misconfigured"
        );

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_the_remote_didnt_change() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        let mut check = GitCheck::open_inner(&local, GitTriggerArgument::Push)?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context)?;
        assert!(!is_pulled);

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_remote_changes() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit
        create_other_repository(&local)?;

        let before_commit_sha = get_last_commit(&local)?;
        let mut check = GitCheck::open_inner(&local, GitTriggerArgument::Push)?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context)?;
        assert!(is_pulled);

        // The pushed file should be pulled
        assert!(Path::new(&format!("{local}/2")).exists());

        // It should set the context keys
        let remote_path = fs::canonicalize(format!("{local}-remote"))?;
        let remote = remote_path.to_str().unwrap();
        let commit_sha = get_last_commit(&local)?;
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

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_return_true_if_the_remote_changes_with_tags() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository, push a new commit and add a tag
        create_other_repository(&local)?;

        let other = format!("{local}-other");
        create_tag(&other, "v0.1.0")?;
        create_commit(&other, "3", "3")?;
        push_all(&other)?;

        let before_commit_sha = get_last_commit(&local)?;
        let mut check = GitCheck::open_inner(&local, GitTriggerArgument::Tag("v*".to_string()))?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context)?;
        assert!(is_pulled);

        // The pushed file should be pulled
        assert!(Path::new(&format!("{local}/2")).exists());

        // The last commit should not be checked out
        assert!(!Path::new(&format!("{local}/3")).exists());

        // Tag should be downloaded
        let tags = get_tags(&local)?;
        assert_eq!(tags, "v0.1.0");

        // It should set the context keys
        let remote_path = fs::canonicalize(format!("{local}-remote"))?;
        let remote = remote_path.to_str().unwrap();
        let commit_sha = get_last_commit(&local)?;
        assert_eq!("tag", context.get("GIT_REF_TYPE").unwrap());
        assert_eq!("refs/tags/v0.1.0", context.get("GIT_REF_NAME").unwrap());
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

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_no_new_tag_with_tags() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        // Create tag with current repository to test if that triggers
        create_empty_repository(&local)?;
        create_tag(&local, "v0.2.0")?;
        push_all(&local)?;

        // Create another repository, push a new commit and add a tag
        create_other_repository(&local)?;

        let other = format!("{local}-other");
        create_commit(&other, "3", "3")?;
        push_all(&other)?;

        let mut check = GitCheck::open_inner(&local, GitTriggerArgument::Tag("v*".to_string()))?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context)?;
        assert!(!is_pulled);

        // The commits should not be downloaded
        assert!(!Path::new(&format!("{local}/2")).exists());
        assert!(!Path::new(&format!("{local}/3")).exists());

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_return_false_if_no_tag_matches_with_tags() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository, push a new commit and add a tag
        create_other_repository(&local)?;

        let other = format!("{local}-other");
        create_tag(&other, "v0.1.0")?;
        create_commit(&other, "3", "3")?;
        push_all(&other)?;

        let mut check =
            GitCheck::open_inner(&local, GitTriggerArgument::Tag("no-match".to_string()))?;
        let mut context: Context = HashMap::new();
        let is_pulled = check.check_inner(&mut context)?;
        assert!(!is_pulled);

        // The commits should not be downloaded
        assert!(!Path::new(&format!("{local}/2")).exists());
        assert!(!Path::new(&format!("{local}/3")).exists());

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_the_working_tree_is_dirty() -> Result<(), Box<dyn Error>> {
        let id = get_random_id();
        let local = format!("test_directories/{id}");

        create_empty_repository(&local)?;

        // Create another repository and push a new commit
        create_other_repository(&local)?;

        // Add uncommited modification to emulate a dirty working tree
        fs::write(format!("{local}/1"), "22")?;

        let mut check = GitCheck::open_inner(&local, GitTriggerArgument::Push)?;
        let mut context: Context = HashMap::new();
        let error = check.check_inner(&mut context).err().unwrap();

        assert!(
            matches!(error, GitError::DirtyWorkingTree),
            "{error:?} should be DirtyWorkingTree"
        );

        // The pushed file should be pulled
        assert!(!Path::new(&format!("{local}/2")).exists());

        let _ = cleanup_repository(&local);

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

        let mut check = GitCheck::open_inner(&local, GitTriggerArgument::Push)?;
        let mut context: Context = HashMap::new();
        let error = check.check_inner(&mut context).err().unwrap();

        assert!(
            matches!(error, GitError::MergeConflict),
            "{error:?} should be MergeConflict"
        );

        let _ = cleanup_repository(&local);

        Ok(())
    }

    #[test]
    #[cfg(unix)]
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

        let mut check: GitCheck = GitCheck::open_inner(&local, GitTriggerArgument::Push)?;
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
        fs::set_permissions(&local, perms)?;

        let _ = cleanup_repository(&local);

        Ok(())
    }
}
