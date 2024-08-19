use super::GitError;
use dirs::home_dir;
use log::{debug, warn};
use std::{
    fs::{create_dir, read_to_string, File},
    io::Write,
    path::PathBuf,
};

// https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/githubs-ssh-key-fingerprints
const GITHUB_FINGERPRINTS: &str = "github.com ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBEmKSENjQEezOmxkZMy7opKgwFB9nkt5YRrYMjNuG5N87uRgg6CLrbo5wAdT/y6v0mKV0U2w0WZ2YB/++Tpockg=
github.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl
github.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCj7ndNxQowgcQnjshcLrqPEiiphnt+VTTvDP6mHBL9j1aNUkY4Ue1gvwnGLVlOhGeYrnZaMgRK6+PKCUXaDbC7qtbW8gIkhL7aGCsOr/C56SJMy/BCZfxd1nWzAOxSDPgVsmerOBYfNqltV9/hWCqBywINIR+5dIg6JTJ72pcEpEjcYgXkE2YEFXV1JHnsKgbLWNlhScqb2UmyRkQyytRLtL+38TGxkxCflmO+5Z8CSSNY7GidjMIZ7Q4zMjA2n1nGrlTDkzwDCsw+wqFPGQA179cnfGWOWRVruj16z6XyvxvjJwbz0wQZ75XK5tKSb7FNyeIEs4TT4jk+S4dhPeAUC5y+bDYirYgM4GC7uEnztnZyaVWQ7B381AK4Qdrwt51ZqExKbQpTUNn+EjqoTwvqNj4kqx5QUCI0ThS/YkOxJCXmPUWZbhjpCg56i+2aB6CmK2JGhn57K5mj0MNdBXA4/WnwH6XoPWJzK5Nyu2zB3nAZp+S5hpQs+p1vN1/wsjk";

// https://docs.gitlab.com/ee/user/gitlab_com/index.html#ssh-host-keys-fingerprints
const GITLAB_FINGERPRINTS: &str = "gitlab.com ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBFSMqzJeV9rUzU4kWitGjeR4PWSa29SPqJ1fVkhtj3Hw9xjLVXVYrU9QlYWrOLXBpQ6KWjbjTDTdDkoohFzgbEY==
gitlab.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAfuCHKVTjquxvt6CM6tdG4SLp1Btn/nOeHHE5UOzRdf
gitlab.com ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCsj2bNKTBSpIYDEGk9KxsGh3mySTRgMtXL583qmBpzeQ+jqCMRgBqB98u3z++J1sKlXHWfM9dyhSevkMwSbhoR8XIq/U0tCNyokEi/ueaBMCvbcTHhO7FcwzY92WK4Yt0aGROY5qX2UKSeOvuP4D6TPqKF1onrSzH9bx9XUf2lEdWT/ia1NEKjunUqu1xOB/StKDHMoX4/OKyIzuS0q/T1zOATthvasJFoPrAjkohTyaDUz2LN5JoH839hViyEG82yB+MjcFV5MU3N1l1QL3cVUCh93xSaua1N85qivl+siMkPGbO5xR/En4iEY6K2XPASUEMaieWVNTRCtJ4S8H+9";

// https://bitbucket.org/site/ssh
const BITBUCKET_FINGERPRINTS: &str = "bitbucket.org ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBPIQmuzMBuKdWeF4+a2sjSSpBK0iqitSQ+5BM9KhpexuGt20JpTVM7u5BDZngncgrqDMbWdxMWWOGtZ9UgbqgZE=
bitbucket.org ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIazEu89wgQZ4bqs3d63QSMzYVa0MuJ2e2gKTKqu+UUO
bitbucket.org ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDQeJzhupRu0u0cdegZIa8e86EG2qOCsIsD1Xw0xSeiPDlCr7kq97NLmMbpKTX6Esc30NuoqEEHCuc7yWtwp8dI76EEEB1VqY9QJq6vk+aySyboD5QF61I/1WeTwu+deCbgKMGbUijeXhtfbxSxm6JwGrXrhBdofTsbKRUsrN1WoNgUa8uqN1Vx6WAJw1JHPhglEGGHea6QICwJOAr/6mrui/oB7pkaWKHj3z7d1IC4KWLtY47elvjbaTlkN04Kc/5LFEirorGYVbt15kAUlqGM65pk6ZBxtaO3+30LVlORZkxOh+LKL/BvbZ/iRNhItLqNyieoQj/uh/7Iv4uyH/cV/0b4WDSd3DptigWq84lJubb9t/DnZlrJazxyDCulTmKdOR7vs9gMTo+uoIrPSb8ScTtvw65+odKAlBj59dhnVp9zd7QUojOpXlL62Aw56U4oO+FALuevvMjiWeavKhJqlR7i5n9srYcrNV7ttmDw7kf/97P5zauIhxcjX+xHv4M=";

/// Setup the known host fingerprints.
///
/// There is no simple way to configure ssh from within libgit2. To make it easier,
/// we err on the side of usability and if there is no `known_hosts` file, we assume
/// that we are in a container and create it with some default fingerprints.
///
/// There is also a flag to add custom host key, in this case we only set that one
/// in the `known_hosts`.
pub fn setup_known_hosts(additional_host: Option<String>) -> Result<(), GitError> {
    let ssh_dir = home_dir().unwrap_or(PathBuf::from("~")).join(".ssh");
    if !ssh_dir.exists() {
        create_dir(&ssh_dir).map_err(|_| GitError::SshConfigFailed)?;
    }

    let known_hosts = ssh_dir.join("known_hosts");

    if let Some(host) = additional_host {
        let mut is_additional_host_found = false;
        if known_hosts.exists() {
            let known_hosts_contents =
                read_to_string(&known_hosts).map_err(|_| GitError::SshConfigFailed)?;
            is_additional_host_found = known_hosts_contents.contains(&host);
        }

        if !is_additional_host_found {
            let mut known_hosts_file = File::options()
                .append(true)
                .create(true)
                .open(&known_hosts)
                .map_err(|_| GitError::SshConfigFailed)?;

            debug!(
                "Host key not found in {}, adding from arguments.",
                known_hosts.to_string_lossy()
            );
            writeln!(known_hosts_file, "{host}").map_err(|_| GitError::SshConfigFailed)?;
        }
    } else if !known_hosts.exists() {
        let mut known_hosts_file =
            File::create(&known_hosts).map_err(|_| GitError::SshConfigFailed)?;

        warn!(
            "There is no {}, creating with default fingerprints.",
            known_hosts.to_string_lossy()
        );
        writeln!(known_hosts_file, "{GITHUB_FINGERPRINTS}")
            .map_err(|_| GitError::SshConfigFailed)?;
        writeln!(known_hosts_file, "{GITLAB_FINGERPRINTS}")
            .map_err(|_| GitError::SshConfigFailed)?;
        writeln!(known_hosts_file, "{BITBUCKET_FINGERPRINTS}")
            .map_err(|_| GitError::SshConfigFailed)?;
    }

    Ok(())
}
