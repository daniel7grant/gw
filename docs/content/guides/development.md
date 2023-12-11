+++
title = "Local development"
weight = 5
+++

# Local development

You can use `gw` to help in local development. You can pull your repository continuously, so in case somebody commits (and there is no conflicts) you can get the newest version. You can also set a notification to see immediately if somebody modified anything.

## Configuration

Simply set the path to your repository:

```sh
gw /path/to/repo -s 'npm run build' -s 'npx pm2 restart'
```

You can use the `notify-send` command to popup notifications on Linux, let's use it to show if somebody commited to our branch.

```sh
gw /path/to/repo -s 'notify-send "$GW_COMMIT_AUTHOR has commited to your branch"'
```