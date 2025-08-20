use super::{
    credentials::{CredentialAuth, CredentialHandler},
    GitError,
};
use git2::{
    AnnotatedCommit, AutotagOption, Config, FetchOptions, Oid, RemoteCallbacks, Repository,
    StatusOptions,
};
use log::{debug, trace};
use std::{collections::HashMap, slice};

pub struct GitRepositoryInformation {
    pub ref_name: String,
    pub branch_name: String,
    pub commit_sha: Oid,
    pub commit_short_sha: String,
    pub remote_name: String,
    pub remote_url: String,
}

/// A directory that is opened as a git repository.
///
/// It is a wrapper around the underlying `git2` [Repository](git2::Repository).
pub struct GitRepository {
    repo: Repository,
    auth: Option<CredentialAuth>,
}

/// Return the 7 characters short hash version for a commit SHA
pub fn shorthash(sha: &Oid) -> String {
    sha.to_string()[0..7].to_string()
}

impl GitRepository {
    /// Open a directory as a GitRepository. Fails if the directory is not a valid git repo.
    pub fn open(directory: &str) -> Result<Self, GitError> {
        let repo = Repository::open(directory).map_err(|err| {
            GitError::NotAGitRepository(String::from(directory), err.message().trim().to_string())
        })?;

        // Do a sanity check to fail instantly if there are any issues
        let git_repo = GitRepository { repo, auth: None };
        git_repo.get_repository_information()?;

        Ok(git_repo)
    }

    pub fn set_auth(&mut self, auth: CredentialAuth) {
        self.auth = Some(auth);
    }

    /// Get information about the current repository, for context and usage in GitRepository
    pub fn get_repository_information(&self) -> Result<GitRepositoryInformation, GitError> {
        let Self { repo, .. } = self;
        let head = repo.head().map_err(|_| GitError::NotOnABranch)?;
        let ref_name = head.name().ok_or(GitError::NotOnABranch)?;
        let commit_sha = head
            .peel_to_commit()
            .map_err(|_| GitError::NotOnABranch)?
            .id();

        let branch_name = head.shorthand().ok_or(GitError::NotOnABranch)?;
        let remote_buf = repo
            .branch_upstream_remote(ref_name)
            .map_err(|_| GitError::NoRemoteForBranch(String::from(branch_name)))?;
        let remote_name = remote_buf
            .as_str()
            .ok_or_else(|| GitError::NoRemoteForBranch(String::from(branch_name)))?;

        let remote = repo
            .find_remote(remote_name)
            .map_err(|_| GitError::NoRemoteForBranch(String::from(branch_name)))?;

        let remote_url = remote
            .url()
            .ok_or(GitError::NoRemoteForBranch(String::from(branch_name)))?;

        Ok(GitRepositoryInformation {
            ref_name: ref_name.to_string(),
            branch_name: branch_name.to_string(),
            commit_short_sha: shorthash(&commit_sha),
            commit_sha,
            remote_url: remote_url.to_string(),
            remote_name: remote_name.to_string(),
        })
    }

    // Inspired from: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
    pub fn fetch(&self) -> Result<AnnotatedCommit<'_>, GitError> {
        let Self { repo, .. } = self;
        let GitRepositoryInformation {
            branch_name,
            remote_name,
            ..
        } = self.get_repository_information()?;

        trace!("Trying to fetch {branch_name} from {remote_name}.");

        let mut remote = repo
            .find_remote(&remote_name)
            .map_err(|_| GitError::NoRemoteForBranch(branch_name.clone()))?;

        // Setup authentication callbacks to fetch the repository
        let mut cb = RemoteCallbacks::new();
        let git_config = Config::open_default().map_err(|_| GitError::ConfigLoadingFailed)?;
        let mut ch = CredentialHandler::new(git_config, self.auth.clone());
        cb.credentials(move |url, username, allowed| {
            ch.try_next_credential(url, username, allowed)
        });

        // Set option to download tags automatically
        let mut opts = FetchOptions::new();
        opts.remote_callbacks(cb);
        opts.download_tags(AutotagOption::Auto);

        // Fetch the remote state
        remote
            .fetch(slice::from_ref(&branch_name), Some(&mut opts), None)
            .map_err(|err| GitError::FetchFailed(err.message().trim().to_string()))?;

        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|err| GitError::FetchFailed(err.message().trim().to_string()))?;
        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|err| GitError::FetchFailed(err.message().trim().to_string()))?;

        trace!(
            "Fetched {remote_name}/{branch_name} successfully to {}.",
            fetch_head
                .peel_to_commit()
                .map(|c| shorthash(&c.id()))
                .unwrap_or("unknown reference".to_string())
        );

        Ok(fetch_commit)
    }

    pub fn check_if_updatable(&self, fetch_commit: &AnnotatedCommit) -> Result<bool, GitError> {
        let Self { repo, .. } = self;
        let (analysis, _) = repo
            .merge_analysis(&[fetch_commit])
            .map_err(|_| GitError::MergeConflict)?;

        if analysis.is_fast_forward() {
            trace!("Fetched commit can be fast forwarded.");
            Ok(true)
        } else if analysis.is_up_to_date() {
            trace!("Fetched commit is up to date.");
            Ok(false)
        } else {
            if analysis.is_unborn() {
                debug!("Fetched commit is not pointing to a valid branch (unborn), failing.");
            } else if analysis.is_normal() {
                debug!("Fetched commit is a merge conflict, failing.");
            }
            Err(GitError::MergeConflict)
        }
    }

    pub fn find_tags(
        &self,
        last_commit_id: Oid,
        pattern: &str,
    ) -> Result<Vec<(String, Oid)>, GitError> {
        let Self { repo, .. } = self;
        let GitRepositoryInformation {
            commit_sha: first_commit_id,
            commit_short_sha: first_commit_short_sha,
            ..
        } = self.get_repository_information()?;

        // Walk from the fetched commit
        let mut revwalk = repo.revwalk().map_err(|_| GitError::TagMatchingFailed)?;
        revwalk.push(last_commit_id).map_err(|_| {
            GitError::FetchFailed("fetched commit is not on this branch".to_string())
        })?;
        trace!(
            "Walking through fetched commits between {}..{}.",
            shorthash(&last_commit_id),
            first_commit_short_sha
        );

        // Collect all tag references beforehand to improve performance
        // If a tag does not point to a valid commit, ignore it
        let tag_names = repo
            .tag_names(Some(pattern))
            .map_err(|_| GitError::TagMatchingFailed)?;
        let tag_commits: HashMap<Oid, String> = tag_names
            .iter()
            .flatten()
            .flat_map(|tag_name| {
                repo.find_reference(&format!("refs/tags/{tag_name}"))
                    .and_then(|tag| tag.peel_to_commit())
                    .map(|tag| (tag.id(), tag_name.to_string()))
            })
            .collect();

        // Go through the list of commits, and register if a commit has a tag pointing to it
        let mut tags = vec![];
        for oid in revwalk {
            let oid = oid.map_err(|_| GitError::TagMatchingFailed)?;

            if oid == first_commit_id {
                break;
            }

            if let Some(tag_name) = tag_commits.get(&oid) {
                debug!("Commit {} has a matching tag: {tag_name}.", shorthash(&oid));
                tags.push((tag_name.clone(), oid));
            }
        }

        if tags.is_empty() {
            debug!("There is no new commit with tag matching \"{pattern}\".");
        }

        // Put it into chronological order
        tags.reverse();

        Ok(tags)
    }

    pub fn pull(&self, commit_id: Oid) -> Result<(), GitError> {
        let Self { repo, .. } = self;
        let GitRepositoryInformation {
            branch_name,
            ref_name,
            ..
        } = self.get_repository_information()?;

        trace!("Pulling {branch_name}.");

        if !repo
            .statuses(Some(StatusOptions::new().include_ignored(false)))
            .map_err(|_| GitError::DirtyWorkingTree)?
            .is_empty()
        {
            return Err(GitError::DirtyWorkingTree);
        }

        let msg = format!("Fast-Forward: Setting {} to id: {}.", ref_name, commit_id);

        let fetch_short = shorthash(&commit_id);
        trace!("Setting {} to id: {}.", ref_name, fetch_short);

        let mut branch_ref = repo
            .find_reference(&ref_name)
            .map_err(|_| GitError::NotOnABranch)?;
        branch_ref
            .set_target(commit_id, &msg)
            .map_err(|_| GitError::FailedSettingHead(fetch_short.to_string()))?;
        repo.set_head(&ref_name)
            .map_err(|_| GitError::FailedSettingHead(fetch_short.to_string()))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|_| GitError::FailedSettingHead(fetch_short.to_string()))?;

        debug!("Checked out {} on branch {}.", fetch_short, branch_name);

        Ok(())
    }
}
