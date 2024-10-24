+++
title = "Authentication"
weight = 2
+++

# Authentication

By default `gw` supports the same authentication methods that `git` does: you can use username-password authentication for HTTPS and ssh-keys with SSH. It uses your platform's credential helpers to find saved authentications. If you want to change these settings, you can use command-line flags.

To debug authentication issues, it is recommended to run `gw` with `-vvv` (tracing mode) to log every credential attempt.

## SSH

For SSH it is recommended to use the ssh-keys that you are already using on your system. If you cloned the repository with a user, you can use the same user to run `gw`. If you are running `gw` in a container, you can mount the whole `.ssh` folder into `/root/.ssh` (`.ssh/known_hosts` is usually needed as well).

> **Note**: If you only running `gw` for a single repository, for improved security use read-only [Deploy keys](#deploy-keys).

By default SSH authentication checks these files for credentials:

-   `.ssh/id_dsa`
-   `.ssh/id_ecdsa`
-   `.ssh/id_ecdsa_sk`
-   `.ssh/id_ed25519`
-   `.ssh/id_ed25519_sk`
-   `.ssh/id_rsa`

If you want to use another file, you can use the `--ssh-key` (or `-i`) option:

```sh
gw /path/to/repo --ssh-key ~/.ssh/id_deploy
```

### SSH known hosts

It is recommended to use the same `.ssh` directory that you used for cloning the repository, because git also requires that the remote's host key appears in `known_hosts`. However, if `gw` doesn't find a `.ssh/known_hosts` file (e.g. in a container), it will create a new one using some common host keys for GitHub, GitLab and Bitbucket. If you are using any of these services, `gw` should work out of the box.

In case you are using another service or self-host your git server, host key checking might fail. Use `--git-known-host` to add a custom host key instead of the default contents to the `.ssh/known_hosts`.

```sh
gw /path/to/repo --git-known-host "codeberg.org ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIVIC02vnjFyL+I4RHfvIGNtOgJMe769VTF1VR4EB3ZB"
```

This is designed for simple operations, if you want to add multiple entries you are better off creating your own `.ssh/known_hosts` file.

### Deploy keys

If you want to use ssh keys with only one repository it is usually better to create a Deploy Key. These are the same as regular ssh keys, but only have pull access to one repository, reducing the attack surface. To get started generate an ssh key:

```
ssh-keygen
```

Both [GitHub](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys) and [GitLab](https://docs.gitlab.com/ee/user/project/deploy_keys) supports Deploy keys by uploading it in the repository settings. Go to **Settings** (or **Settings** > **Repository** on GitLab) and enter **Deploy keys** and press **Add new deploy key**. Copy the content of the public key into the **Key** textarea and save it. `gw` never writes, so pull access is enough.

If you used a non-default path, set it with `--ssh-key`:

```sh
gw /path/to/repo --ssh-key ~/.ssh/id_deploy
```

## Https

Even though it is less common on servers, you can also use HTTPS for pulling repositories. By default `gw` will check credential helpers to extract username and passwords. If you want to set a username and password manually you can use the `--git-username` and `--git-token` fields.

> **Note**: **Never** use your password as the `--git-token`, always use read-only [repository-level tokens](#repository-level-tokens) instead.

```sh
gw /path/to/repo --git-username username --git-token f7818t23fb1amsc
```

If you are going this route, be careful to never leak your credentials and only use tokens with minimal privileges!

### Repository-level tokens

Similarly to Deploy keys, it is recommended to only allow tokens access to a single repository. For this you can use [GitHub's Fine-grained personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token) or [GitLab's Deploy tokens](https://docs.gitlab.com/ee/user/project/deploy_tokens/).

#### Set up fine-grained access tokens in GitHub

In GitHub, there are no repository scoped tokens, but you can emulate one with [Fine-grained personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token).

Click on your avatar and go to **Settings** > **Developer Settings** > **Personal access tokens** > **Fine-grained tokens**. Click on **Generate new token**, enter a name and set an expiration longer into the future. Select the repositories that this token should have access under **Only select repositories**. Under **Repository Permissions** select **Access: read-only** for **Contents**. Copy this token and use it with your GitHub username:

```sh
gw /path/to/repo --git-username octocat --git-token github_pat_11AD...
```

#### Set up deploy tokens in GitLab

In GitLab there are [Deploy tokens](https://docs.gitlab.com/ee/user/project/deploy_tokens/), that are access tokens scoped to a repository.

Go to the project's **Settings** > **Repository** > **Deploy tokens** and click on **Add token**. Fill out the username and check `read_repository` to be able to pull commits. Copy the username and the token to use it:

```sh
gw /path/to/repo --git-username git_token --git-token gldt-...
```
