use super::{credentials::CredentialHandler, GitError};
use git2::{AnnotatedCommit, AutotagOption, Config, FetchOptions, RemoteCallbacks, Repository};
use log::{debug, trace};

pub struct GitRepository {
    repo: Repository,
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

        trace!("Trying to fetch {branch_name} from {remote_name}.");

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

        let mut opts = FetchOptions::new();
        opts.remote_callbacks(cb);
        opts.download_tags(AutotagOption::Auto);

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
        let head = repo.head().map_err(|_| GitError::NotOnABranch)?;
        let (analysis, _) = repo
            .merge_analysis(&[fetch_commit])
            .map_err(|_| GitError::MergeConflict)?;

        if analysis.is_fast_forward() {
            trace!("Fetched commit can be fast forwarded.");
            Ok(true)
        } else if analysis.is_up_to_date() {
            trace!("Fetched commit is up to date.");
            if let Some(head_id) = head.target() {
                debug!(
                    "Comparing fetch commit and HEAD ({} - {}).",
                    &fetch_commit.id().to_string()[0..7],
                    &head_id.to_string()[0..7]
                );
                if fetch_commit.id() != head_id {
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
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
        let head = repo.head().map_err(|_| GitError::NotOnABranch)?;
        let branch_name = head.shorthand().ok_or(GitError::NotOnABranch)?;

        trace!("Pulling {branch_name}.");

        // TODO: Only fetch if the repository is not dirty

        let branch_refname = format!("refs/heads/{}", branch_name);
        let branch_ref = repo
            .find_reference(&branch_refname)
            .map_err(|_| GitError::NotOnABranch)?;

        let name = branch_ref
            .name()
            .ok_or_else(|| GitError::NoRemoteForBranch(String::from(branch_name)))?;
        let msg = format!(
            "Fast-Forward: Setting {} to id: {}.",
            name,
            fetch_commit.id()
        );

        trace!("Setting {} to id: {}.", name, fetch_commit.id().to_string()[0..7].to_string());

        let mut branch_ref = repo
            .find_reference(&branch_refname)
            .map_err(|_| GitError::NotOnABranch)?;
        let fetch_id = fetch_commit.id();
        branch_ref
            .set_target(fetch_id, &msg)
            .map_err(|_| GitError::FailedSettingHead(fetch_id.to_string()[0..7].to_string()))?;
        repo.set_head(name)
            .map_err(|_| GitError::FailedSettingHead(fetch_id.to_string()[0..7].to_string()))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|_| GitError::FailedSettingHead(fetch_id.to_string()[0..7].to_string()))?;

        trace!("Checked out {} on branch {}.", fetch_commit.id(), branch_name);

        Ok(true)
    }
}
