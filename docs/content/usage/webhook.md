+++
title = "Webhook server"
weight = 6
+++

# Webhook server

By default `gw` checks for updates every minute. Depending on your usecase it can be too slow or too often. If you only want to pull updates when a push happens, git servers (GitHub, GitLab or any other) usually have options to send a HTTP request to your `gw` service (webhook). `gw` can handle webhooks with a built-in web server.

## Usage

To enable the webhook server, you can use the `--http` option. Most of the time you want to allow external connections, so to set to a high port (for example `10101`), you can use:

```sh
gw . -v --http 0.0.0.0:10101
```

If you call this endpoint with any method on any URL, it will trigger a check for updates. To test this, you can use `curl`:

```sh
curl http://localhost:10101
```

The `curl` output should print `OK` and the `gw` logs should include lines that show that it was updated:

```sh
$ gw . -v --http 0.0.0.0:10101
# ...
2024-03-10T16:52:51.531Z DEBUG [gw_bin::triggers::http] Received request on GET /
2024-03-10T16:52:52.055Z DEBUG [gw_bin::checks::git::repository] Checked out 5e25714 on branch main.
2024-03-10T16:52:52.055Z DEBUG [gw_bin::start] There are updates, pulling.
```

If you want to disable the scheduled checks altogether and rely on the webhooks, you can set the schedule duration (`-d` flag) to zero seconds:

```sh
gw . -v --http 0.0.0.0:10101 -d 0s
```

## Setup webhooks

Exposing a port is only one half of the problem, you also have to set the webhooks up with your git server. For this you will need a public IP or a domain name, which will be in the `$DOMAIN` variable in these examples.

> **Warning:** if you can configure, you should setup your reverse proxy in front of the port to avoid exposing externally.

### GitHub

For GitHub, you have to have administrator access to the repository. Navigate to **Settings > Webhooks**, and click to **Add webhook**. Fill the **Payload URL** with your `$DOMAIN` (make sure to add the `http://` protocol and the port) and select **application/json** for **Content Type**. Save this webhook to activate.

> **Note**: Secrets are currently not supported.

![You have to setup the payload URL to be http://$DOMAIN:10101 on GitHub.](/webhook-github.png)

On save, the webhook should send a `ping` event to `gw`. If you click into new webhook, to **Recent deliveries** you can see this event.

![A ping event has been delivered to the server.](/webhook-github-deliveries.png)

A `POST /` request will also appear in the `gw` logs, assuming debug logging was enabled:

```sh
$ gw . -v --http 0.0.0.0:10101
# ...
2024-03-10T17:18:24.424Z DEBUG [gw_bin::triggers::http] Received request on POST /
2024-03-10T17:18:24.567Z DEBUG [gw_bin::start] There are no updates.
```

### GitLab

For GitLab, you have to have Maintainer access to the repository. Navigate to **Settings > Webhooks**, and click to **Add new webhook**. Fill the **URL** with your `$DOMAIN` (make sure to add the `http://` protocol and the port) and check the **Trigger** to **Push events** , you can filter it for example to only trigger on the `main` branch. If you are using `http`, you should disable SSL verification. Save this webhook to activate.

![You have to setup the URL to be http://$DOMAIN:10101 on GitLab and check Push events.](/webhook-gitlab.png)

To test this webhook, you can click **Test > Push events** next to the name. GitLab should show a message that **Hook executed successfully: HTTP 200**, and you can find a `POST /` request in the `gw` logs, assuming debug logging was enabled:

```sh
$ gw . -v --http 0.0.0.0:10101
# ...
2024-03-10T17:58:28.919Z DEBUG [gw_bin::triggers::http] Received request on POST /
2024-03-10T17:58:29.052Z DEBUG [gw_bin::start] There are no updates.
```
