+++
title = "Systemd unit"
weight = 4
+++

# Systemd unit

If you just want to run `gw` process in the background without installing anything, you can use `systemctl`. You only have to create a systemd unit and `start` and `enable` it.

> **Note**: by default systemd units run as root, so make sure to set up the necessary authentication (e.g. SSH keys) with the root user as well
> (or use a `User` directive or [user systemd unit](#user-systemd-unit)).
> You can test this by entering the directory with the root user and running `git pull` or `gw -vv .`.

## Usage

To create a new unit you have to create a new unit file at the default systemd unit location, usually `/etc/systemd/system`.

You can change this example systemd unit to your use-case and copy it under `/etc/systemd/system/gw.service`:

```ini
# /etc/systemd/system/gw.service
[Unit]
Description=Watch git repository at /path/to/repo
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/bin/gw -v /path/to/repo -s 'echo ran from systemctl unit'
Restart=always
# run as a non-root user (recommended)
User=myuser

[Install]
WantedBy=default.target
```

To reload the systemd unit database, you have to run `daemon-reload`:

```sh
systemctl daemon-reload
```

With this you should see a new `gw.service` unit. You can start this with `systemctl start`:

```sh
systemctl start gw
```

If you want to start this every time your server boots up, you can run `systemctl enable`:

```sh
systemctl enable gw
```

To see if your unit is running you check the status, or read the logs with `journalctl`:

```sh
systemctl status gw
journalctl -fu gw
```

For a more complicated example, check out the [docker-compose systemd unit](/guides/docker-compose#systemd-unit).

### User systemd unit

Most of the time the git configuration is only set up for some users, so it might make sense to run `gw` as a user systemd unit. You can do it, but you have to be careful with some things.

The main issue with user services is that they are bound to the user session, so if you log out from the SSH, all started units will end. You can enable [lingering](https://wiki.archlinux.org/title/Systemd/User#Automatic_start-up_of_systemd_user_instances) with `loginctl` to keep the systemd units running after logout:

```
loginctl enable-linger
```

After you set up lingering, you can create a similar systemd unit except under `~/.config/systemd/user/`:

```ini
# /home/myuser/.config/systemd/user/gw.service
[Unit]
Description=Watch git repository at /path/to/repo
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/bin/gw /path/to/repo -s 'echo ran from systemctl unit'
Restart=always

[Install]
WantedBy=default.target
```

The same commands should work as above but with adding `--user` after `systemctl`. So to enable this unit above, you can start:

```sh
systemctl --user daemon-reload
systemctl --user start gw
systemctl --user enable gw
systemctl --user status gw
```

If you want to check the logs with `journalctl`, make sure to add your user to the `systemd-journal` group (requires root privileges):

```sh
sudo usermod -aG $USER systemd-journal
```

After this, you can read the logs of your user services:

```sh
journalctl -f --user-unit gw
```

User services can be a good way to use `gw` if you don't have or don't want to use root privileges, while still being able to use an automatic deployment workflow.
