use std::fmt::Debug;

use super::credentials::CredentialHandler;
use git2::{AnnotatedCommit, AutotagOption, FetchOptions, RemoteCallbacks, Repository};
use thiserror::Error;

pub struct GitRepository {
    repo: Repository,
}

/// Custom error describing the error cases for the GitCheck.
#[derive(Debug, Error)]
pub enum GitError {
    /// The directory is not a valid git repository.
    #[error("{0} does not exist, user don't have access or not a git repository")]
    NotAGitRepository(String),
    /// There is no branch in the repository currently. It can be a repository
    /// without any branch, or checked out on a commit.
    #[error("we are currently not on a branch")]
    NotOnABranch,
    /// There is no remote for the current branch. This usually because the branch hasn't been pulled.
    #[error("branch {0} doesn't have a remote")]
    NoRemoteForBranch(String),
    /// There are changes in the directory, avoiding pulling. This is a safety mechanism to avoid pulling
    /// over local changes, to not overwrite anything important.
    #[error("there are changes in the directory, not pulling")]
    DirtyWorkingTree,
    /// Cannot fetch the current branch. This is possibly a network failure.
    #[error("cannot fetch, might be a network error")]
    FetchFailed,
    /// Cannot pull updates to the current branch. This means either the merge analysis failed
    /// or there is a merge conflict.
    #[error("cannot update branch, there is likely a merge conflict")]
    MergeConflict,
    /// Cannot set the HEAD to the fetch commit.
    #[error("could not set HEAD to fetch commit {0}, might be permission issues")]
    FailedSettingHead(String),
}

impl GitRepository {
    pub fn open(directory: &str) -> Result<Self, GitError> {
        let repo = Repository::open(directory)
            .map_err(|_| GitError::NotAGitRepository(String::from(directory)))?;

        Ok(GitRepository { repo })
    }

    // Inspired from: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
    pub fn fetch(&self) -> Result<AnnotatedCommit, GitError> {
        let Self { repo, .. } = self;
        let head = repo.head().map_err(|_| GitError::NotOnABranch)?;
        let branch_name = head.shorthand().ok_or(GitError::NotOnABranch)?;
        let branch_name_ref = head.name().ok_or(GitError::NotOnABranch)?;
        let remote_buf = repo
            .branch_upstream_remote(branch_name_ref)
            .map_err(|_| GitError::NoRemoteForBranch(String::from(branch_name)))?;
        let remote_name = remote_buf
            .as_str()
            .ok_or_else(|| GitError::NoRemoteForBranch(String::from(branch_name)))?;

        let mut remote = repo
            .find_remote(remote_name)
            .map_err(|_| GitError::NoRemoteForBranch(String::from(branch_name)))?;

        let mut cb = RemoteCallbacks::new();
        let git_config = git2::Config::open_default().unwrap();
        let mut ch = CredentialHandler::new(git_config);
        cb.credentials(move |url, username, allowed| {
            ch.try_next_credential(url, username, allowed)
        });

        let mut opts = FetchOptions::new();
        opts.remote_callbacks(cb);
        opts.download_tags(AutotagOption::All);

        remote
            .fetch(&[branch_name], Some(&mut opts), None)
            .map_err(|_| GitError::FetchFailed)?;

        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|_| GitError::FetchFailed)?;
        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|_| GitError::FetchFailed)?;

        Ok(fetch_commit)
    }

    pub fn check_if_updatable(&self, fetch_commit: &AnnotatedCommit) -> Result<bool, GitError> {
        let Self { repo, .. } = self;
        let (analysis, _) = repo
            .merge_analysis(&[fetch_commit])
            .map_err(|_| GitError::MergeConflict)?;

        if analysis.is_fast_forward() {
            Ok(true)
        } else if analysis.is_up_to_date() {
            Ok(false)
        } else {
            Err(GitError::MergeConflict)
        }
    }

    pub fn pull(&self, fetch_commit: &AnnotatedCommit) -> Result<bool, GitError> {
        let Self { repo, .. } = self;
        let head = repo.head().map_err(|_| GitError::NotOnABranch)?;
        let branch_name = head.shorthand().ok_or(GitError::NotOnABranch)?;

        // TODO: Only fetch if the repository is not dirty

        let branch_refname = format!("refs/heads/{}", branch_name);
        let branch_ref = repo
            .find_reference(&branch_refname)
            .map_err(|_| GitError::NotOnABranch)?;

        let name = branch_ref
            .name()
            .ok_or_else(|| GitError::NoRemoteForBranch(String::from(branch_name)))?;
        let msg = format!(
            "Fast-Forward: Setting {} to id: {}",
            name,
            fetch_commit.id()
        );

        let mut branch_ref = repo
            .find_reference(&branch_refname)
            .map_err(|_| GitError::NotOnABranch)?;
        let fetch_id = fetch_commit.id();
        branch_ref
            .set_target(fetch_id, &msg)
            .map_err(|_| GitError::FailedSettingHead(fetch_id.to_string()))?;
        repo.set_head(name)
            .map_err(|_| GitError::FailedSettingHead(fetch_id.to_string()))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|_| GitError::FailedSettingHead(fetch_id.to_string()))?;
        Ok(true)
    }
}
