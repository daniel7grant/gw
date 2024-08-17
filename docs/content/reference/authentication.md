+++
title = "Authentication"
weight = 1
+++

# Authentication

By default `gw` supports the same authentication methods that `git` does: you can use username-password authentication for HTTPS and ssh-keys with SSH. It uses your platform's credential helpers to find saved authentications. If you want to change these settings, you can use command-line flags.

To debug authentication issues, it is recommended to run `gw` with `-vvv` (tracing mode) to log every credential attempt.

## SSH

For SSH it is recommended to use the ssh-keys that you are already using on your system. If you cloned the repository with a user, you can use the same user to run `gw`. If you are running `gw` in a container, you can mount the whole `.ssh` folder into `/root/.ssh` (`.ssh/known_hosts` is usually needed as well).

> Note: If you only running `gw` for a single repository, for improved security use read-only Deploy Keys with [GitHub](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys) or [GitLab](https://docs.gitlab.com/ee/user/project/deploy_keys).

By default SSH authentication checks these files for credentials:

- `.ssh/id_dsa`
- `.ssh/id_ecdsa`
- `.ssh/id_ecdsa_sk`
- `.ssh/id_ed25519`
- `.ssh/id_ed25519_sk`
- `.ssh/id_rsa`

If you want to use another file, you can use the `--ssh-key` (or `-i`) option:

```sh
gw /path/to/repo --ssh-key ~/.ssh/id_deploy
```

## Https

Even though it is less common on servers, you can also use HTTPS for pulling repositories. By default `gw` will check credential helpers to extract username and passwords. If you want to set a username and password manually you can use the `--git-username` and `--git-token` fields.

> Note: **Never** use your password as the `--git-token`, always use read-only tokens: either [GitHub's Fine-grained personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token) or [GitLab's Deploy Tokens](https://docs.gitlab.com/ee/user/project/deploy_tokens/).

```sh
gw /path/to/repo --git-username octocat --git-token github_pat_11AD...
```

If you are going this route, be careful to never leak your credentials and only use tokens with minimal privileges!
