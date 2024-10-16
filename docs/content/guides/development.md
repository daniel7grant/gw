+++
title = "Local development"
weight = 6
+++

# Local development

You can use `gw` to help in local development. You can pull your repository continuously, so in case somebody commits (and there is no conflicts) you can get the newest version. You can also set a notification to see immediately if somebody modified anything.

> I don't recommend working on the same branch with multiple people, but sometimes it happens.

## Configuration

Simply set the path to your repository:

```sh
gw /path/to/repo
```

You can use the `notify-send` command to popup notifications on Linux, let's use it to show if somebody commited to our branch.

```sh
gw /path/to/repo -s 'notify-send "There are new commits on your branch!"'
```