// Credential funtion graciously lifted from https://github.com/davidB/git2_credentials
// The goal is to remove every extra feature (e.g. interactive usage, config parsing with pest)

use dirs::home_dir;
use log::{trace, warn};
use std::path::PathBuf;

pub use git2;

#[derive(Debug, Clone)]
pub enum CredentialAuth {
    Ssh(String),
    Https(String, String),
}

pub struct CredentialHandler {
    username_attempts_count: usize,
    username_candidates: Vec<String>,
    ssh_attempts_count: usize,
    ssh_key_candidates: Vec<std::path::PathBuf>,
    https_credentials: Option<(String, String)>,
    cred_helper_bad: Option<bool>,
    cfg: git2::Config,
}

// implemention based on code & comment from cargo
// https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/utils.rs#L415-L628
// License APACHE
// but adapted to not use wrapper over function like withXxx(FnMut), a more OO approach
impl CredentialHandler {
    pub fn new(cfg: git2::Config, auth: Option<CredentialAuth>) -> Self {
        // Force using https credentials if given
        if let Some(CredentialAuth::Https(username, password)) = auth {
            return CredentialHandler {
                username_attempts_count: 0,
                username_candidates: vec!["git".to_string()],
                ssh_attempts_count: 0,
                ssh_key_candidates: vec![],
                cred_helper_bad: None,
                https_credentials: Some((username, password)),
                cfg,
            };
        }

        // Generate a list of available keys
        let ssh_keys = if let Some(CredentialAuth::Ssh(path)) = auth {
            vec![PathBuf::from(path)]
        } else {
            let home = home_dir().unwrap_or(PathBuf::from("~"));
            vec![
                home.join(".ssh/id_dsa"),
                home.join(".ssh/id_ecdsa"),
                home.join(".ssh/id_ecdsa_sk"),
                home.join(".ssh/id_ed25519"),
                home.join(".ssh/id_ed25519_sk"),
                home.join(".ssh/id_rsa"),
            ]
        };

        let ssh_key_candidates: Vec<PathBuf> = ssh_keys
            .into_iter()
            .filter(|key_path| key_path.exists())
            .collect();

        CredentialHandler {
            username_attempts_count: 0,
            username_candidates: vec!["git".to_string()],
            ssh_attempts_count: 0,
            ssh_key_candidates,
            cred_helper_bad: None,
            https_credentials: None,
            cfg,
        }
    }

    /// Prepare the authentication callbacks for cloning a git repository.
    ///
    /// The main purpose of this function is to construct the "authentication
    /// callback" which is used to clone a repository. This callback will attempt to
    /// find the right authentication on the system (maybe with user input) and will
    /// guide libgit2 in doing so.
    ///
    /// The callback is provided `allowed` types of credentials, and we try to do as
    /// much as possible based on that:
    ///
    /// - Prioritize SSH keys from the local ssh agent as they're likely the most
    ///   reliable. The username here is prioritized from the credential
    ///   callback, then from whatever is configured in git itself, and finally
    ///   we fall back to the generic user of `git`. If no ssh agent try to use
    ///   the default key ($HOME/.ssh/id_rsa, $HOME/.ssh/id_ed25519)
    ///
    /// - If a username/password is allowed, then we fallback to git2-rs's
    ///   implementation of the credential helper. This is what is configured
    ///   with `credential.helper` in git, and is the interface for the macOS
    ///   keychain, for example. Else ask (on ui) the for username and password.
    ///
    /// - After the above two have failed, we just kinda grapple attempting to
    ///   return *something*.
    ///
    /// If any form of authentication fails, libgit2 will repeatedly ask us for
    /// credentials until we give it a reason to not do so. To ensure we don't
    /// just sit here looping forever we keep track of authentications we've
    /// attempted and we don't try the same ones again.
    pub fn try_next_credential(
        &mut self,
        url: &str,
        username: Option<&str>,
        allowed: git2::CredentialType,
    ) -> Result<git2::Cred, git2::Error> {
        // libgit2's "USERNAME" authentication actually means that it's just
        // asking us for a username to keep going. This is currently only really
        // used for SSH authentication and isn't really an authentication type.
        // The logic currently looks like:
        //
        //      let user = ...;
        //      if (user.is_null())
        //          user = callback(USERNAME, null, ...);
        //
        //      callback(SSH_KEY, user, ...)
        //
        // So if we're being called here then we know that (a) we're using ssh
        // authentication and (b) no username was specified in the URL that
        // we're trying to clone. We need to guess an appropriate username here,
        // but that may involve a few attempts.
        // (FIXME) Unfortunately we can't switch
        // usernames during one authentication session with libgit2, so to
        // handle this we bail out of this authentication session after setting
        // the flag `ssh_username_requested`, and then we handle this below.
        if allowed.contains(git2::CredentialType::USERNAME) {
            // debug_assert!(username.is_none());
            let idx = self.username_attempts_count;
            self.username_attempts_count += 1;
            return match self.username_candidates.get(idx).map(|s| &s[..]) {
                Some(s) => git2::Cred::username(s),
                _ => Err(git2::Error::from_str("no more username to try")),
            };
        }

        // An "SSH_KEY" authentication indicates that we need some sort of SSH
        // authentication. This can currently either come from the ssh-agent
        // process or from a raw in-memory SSH key.
        //
        // If we get called with this then the only way that should be possible
        // is if a username is specified in the URL itself (e.g., `username` is
        // Some), hence the unwrap() here. We try custom usernames down below.
        if allowed.contains(git2::CredentialType::SSH_KEY) {
            // If ssh-agent authentication fails, libgit2 will keep
            // calling this callback asking for other authentication
            // methods to try. Make sure we only try ssh-agent once.
            self.ssh_attempts_count += 1;
            let u = username.unwrap_or("git");
            return if self.ssh_attempts_count == 1 {
                trace!("Trying ssh-key from agent with username {u}.");
                git2::Cred::ssh_key_from_agent(u)
            } else {
                let candidate_idx = self.ssh_attempts_count - 2;
                if candidate_idx < self.ssh_key_candidates.len() {
                    let key = self.ssh_key_candidates.get(candidate_idx);
                    match key {
                        // try without passphrase
                        Some(k) => {
                            trace!("Trying ssh-key {} without passphrase.", k.to_string_lossy());
                            git2::Cred::ssh_key(u, None, k, None)
                        }
                        None => Err(git2::Error::from_str(
                            "failed ssh authentication for repository",
                        )),
                    }
                } else {
                    if self.ssh_key_candidates.is_empty() {
                        warn!("There are no ssh-keys in ~/.ssh, run ssh-keygen or mount your .ssh directory.");
                    }
                    Err(git2::Error::from_str(
                        "no ssh-key found that can authenticate to your repository",
                    ))
                }
            };
        }

        // Sometimes libgit2 will ask for a username/password in plaintext.
        //
        // If ssh-agent authentication fails, libgit2 will keep calling this
        // callback asking for other authentication methods to try. Check
        // cred_helper_bad to make sure we only try the git credentail helper
        // once, to avoid looping forever.
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
            && self.cred_helper_bad.is_none()
        {
            if let Some((username, password)) = &self.https_credentials {
                trace!("Trying username-password from command line argument {username}.");
                return git2::Cred::userpass_plaintext(username, password);
            }

            trace!("Trying username-password authentication from credential helper.");
            let r = git2::Cred::credential_helper(&self.cfg, url, username);
            self.cred_helper_bad = Some(r.is_err());
            return r;
        }

        // I'm... not sure what the DEFAULT kind of authentication is, but seems
        // easy to support?
        if allowed.contains(git2::CredentialType::DEFAULT) {
            return git2::Cred::default();
        }

        // Stop trying
        trace!("There are not authentication available.");
        Err(git2::Error::from_str("no valid authentication available"))
    }
}
