use super::credentials::CredentialHandler;
use crate::Result;
use git2::{
    AnnotatedCommit, AutotagOption, Error as GitError, FetchOptions, RemoteCallbacks, Repository,
};

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open(directory: &str) -> Result<Self> {
        let repo = Repository::open(directory).map_err(|err| err.to_string())?;

        Ok(GitRepository { repo })
    }

    // Inspired from: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
    pub fn fetch(&self) -> Result<AnnotatedCommit> {
        let Self { repo, .. } = self;
        let head = repo.head()?;
        let branch_name = head
            .shorthand()
            .ok_or(GitError::from_str("We are currently not on a branch."))?;
        let branch_name_ref = head
            .name()
            .ok_or(GitError::from_str("We are currently not on a branch."))?;
        let remote_buf = repo.branch_upstream_remote(branch_name_ref)?;
        let remote_name = remote_buf
            .as_str()
            .ok_or(GitError::from_str("This branch doesn't have a remote."))?;

        let mut remote = repo.find_remote(remote_name)?;

        let mut cb = RemoteCallbacks::new();
        let git_config = git2::Config::open_default().unwrap();
        let mut ch = CredentialHandler::new(git_config);
        cb.credentials(move |url, username, allowed| {
            ch.try_next_credential(url, username, allowed)
        });

        let mut opts = FetchOptions::new();
        opts.remote_callbacks(cb);
        opts.download_tags(AutotagOption::All);

        remote.fetch(&[branch_name], Some(&mut opts), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        Ok(fetch_commit)
    }

    pub fn check_if_updatable(&self, fetch_commit: &AnnotatedCommit) -> Result<bool> {
        let Self { repo, .. } = self;
        let (analysis, _) = repo.merge_analysis(&[fetch_commit])?;

        if analysis.is_fast_forward() {
            Ok(true)
        } else if analysis.is_up_to_date() {
            Ok(false)
        } else {
            Err(Box::<dyn std::error::Error>::from(GitError::from_str(
                "Could not update branch. Possibly there is a merge conflict.",
            )))
        }
    }

    pub fn pull(&self, fetch_commit: &AnnotatedCommit) -> Result<bool> {
        let Self { repo, .. } = self;
        let head = repo.head()?;
        let branch_name = head
            .shorthand()
            .ok_or(GitError::from_str("We are currently not on a branch."))?;

        // TODO: Only fetch if the repository is dirty

        let branch_refname = format!("refs/heads/{}", branch_name);
        let branch_ref = repo.find_reference(&branch_refname)?;

        let name = branch_ref
            .name()
            .ok_or(GitError::from_str("Remote branch name is invalid."))?;
        let msg = format!(
            "Fast-Forward: Setting {} to id: {}",
            name,
            fetch_commit.id()
        );

        let mut branch_ref = repo.find_reference(&branch_refname)?;
        branch_ref.set_target(fetch_commit.id(), &msg)?;
        repo.set_head(name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok(true)
    }
}
