use super::{credentials::CredentialHandler, GitError};
use git2::{
    AnnotatedCommit, AutotagOption, Config, FetchOptions, RemoteCallbacks, Repository,
    StatusOptions,
};
use log::{debug, trace};

pub enum GitRepositoryInformation {
    Branch {
        ref_type: String,
        ref_name: String,
        branch_name: String,
        commit_sha: String,
        commit_short_sha: String,
        remote_name: String,
        remote_url: String,
    },
    Reference {
        ref_type: String,
        ref_name: String,
        commit_sha: String,
        commit_short_sha: String,
    },
}

/// A directory that is opened as a git repository.
///
/// It is a wrapper around the underlying `git2` [Repository](git2::Repository).
pub struct GitRepository {
    repo: Repository,
}

/// Return the 7 characters short hash version for a commit SHA
pub fn shorthash(sha: &str) -> String {
    sha[0..7].to_string()
}

impl GitRepository {
    /// Open a directory as a GitRepository. Fails if the directory is not a valid git repo.
    pub fn open(directory: &str) -> Result<Self, GitError> {
        let mut cfg = Config::open_default().map_err(|_| GitError::ConfigLoadingFailed)?;
        cfg.set_str("safe.directory", directory)
            .map_err(|_| GitError::ConfigLoadingFailed)?;

        let repo = Repository::open(directory)
            .map_err(|_| GitError::NotAGitRepository(String::from(directory)))?;

        Ok(GitRepository { repo })
    }

    /// Get information about the current repository, for context and usage in GitRepository
    pub fn get_repository_information(&self) -> Result<GitRepositoryInformation, GitError> {
        let Self { repo, .. } = self;
        let head = repo.head().map_err(|_| GitError::NotOnABranch)?;
        let ref_name = head.name().ok_or(GitError::NotOnABranch)?;
        let commit_sha = head
            .peel_to_commit()
            .map_err(|_| GitError::NotOnABranch)?
            .id()
            .to_string();

        let branch_name = head.shorthand();
        if let Some(branch_name) = branch_name {
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

            Ok(GitRepositoryInformation::Branch {
                ref_type: "branch".to_string(),
                ref_name: ref_name.to_string(),
                branch_name: branch_name.to_string(),
                commit_short_sha: shorthash(&commit_sha),
                commit_sha,
                remote_url: remote_url.to_string(),
                remote_name: remote_name.to_string(),
            })
        } else {
            Ok(GitRepositoryInformation::Reference {
                ref_type: "reference".to_string(),
                ref_name: ref_name.to_string(),
                commit_short_sha: shorthash(&commit_sha),
                commit_sha,
            })
        }
    }

    // Inspired from: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
    pub fn fetch(&self) -> Result<AnnotatedCommit, GitError> {
        let Self { repo, .. } = self;
        let (branch_name, remote_name) = match self.get_repository_information()? {
            GitRepositoryInformation::Branch {
                branch_name,
                remote_name,
                ..
            } => Ok((branch_name, remote_name)),
            GitRepositoryInformation::Reference { .. } => Err(GitError::NotOnABranch),
        }?;

        trace!("Trying to fetch {branch_name} from {remote_name}.");

        let mut remote = repo
            .find_remote(&remote_name)
            .map_err(|_| GitError::NoRemoteForBranch(branch_name.clone()))?;

        // Setup authentication callbacks to fetch the repository
        let mut cb = RemoteCallbacks::new();
        let git_config = Config::open_default().map_err(|_| GitError::ConfigLoadingFailed)?;
        let mut ch = CredentialHandler::new(git_config);
        cb.credentials(move |url, username, allowed| {
            trace!("Trying credential {username:?} for {url}.");
            let try_cred = ch.try_next_credential(url, username, allowed);
            if try_cred.is_err() {
                debug!("Cannot authenticate with {url}.");
            }
            try_cred
        });

        // Set option to download tags automatically
        let mut opts = FetchOptions::new();
        opts.remote_callbacks(cb);
        opts.download_tags(AutotagOption::Auto);

        // Fetch the remote state
        remote
            .fetch(&[branch_name], Some(&mut opts), None)
            .map_err(|_| GitError::FetchFailed)?;

        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|_| GitError::FetchFailed)?;
        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|_| GitError::FetchFailed)?;

        trace!(
            "Fetched successfully to {}.",
            fetch_head
                .peel_to_commit()
                .map(|c| c.id().to_string()[0..7].to_string())
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

    pub fn pull(&self, fetch_commit: &AnnotatedCommit) -> Result<bool, GitError> {
        let Self { repo, .. } = self;
        let (branch_name, ref_name) = match self.get_repository_information()? {
            GitRepositoryInformation::Branch {
                branch_name,
                ref_name,
                ..
            } => Ok((branch_name, ref_name)),
            GitRepositoryInformation::Reference { .. } => Err(GitError::NotOnABranch),
        }?;

        trace!("Pulling {branch_name}.");

        if !repo
            .statuses(Some(StatusOptions::new().include_ignored(false)))
            .map_err(|_| GitError::DirtyWorkingTree)?
            .is_empty()
        {
            return Err(GitError::DirtyWorkingTree);
        }

        let msg = format!(
            "Fast-Forward: Setting {} to id: {}.",
            ref_name,
            fetch_commit.id()
        );

        let fetch_short = shorthash(&fetch_commit.id().to_string());
        trace!("Setting {} to id: {}.", ref_name, fetch_short);

        let mut branch_ref = repo
            .find_reference(&ref_name)
            .map_err(|_| GitError::NotOnABranch)?;
        let fetch_id = fetch_commit.id();
        branch_ref
            .set_target(fetch_id, &msg)
            .map_err(|_| GitError::FailedSettingHead(fetch_short.to_string()))?;
        repo.set_head(&ref_name)
            .map_err(|_| GitError::FailedSettingHead(fetch_short.to_string()))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|_| GitError::FailedSettingHead(fetch_short.to_string()))?;

        debug!("Checked out {} on branch {}.", fetch_short, branch_name);

        Ok(true)
    }
}
