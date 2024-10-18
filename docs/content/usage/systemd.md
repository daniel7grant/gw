+++
title = "Systemd unit"
weight = 4
+++

# Systemd unit

If you just want to run `gw` process in the background without installing anything, you can use `systemctl`. You only have to create a systemd unit and `start` and `enable` it.

## Usage

To create a new unit you have to create a new unit file at the default systemd unit location, usually `/etc/systemd/system`.

You can change this example systemd unit to your use-case and copy it under `/etc/systemd/system/gw.service`:

```ini
[Unit]
Description=Watch git repository at /path/to/repo
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/bin/gw /path/to/repo -s 'echo ran from systemctl unit'
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
systemctl start gw.service
```

If you want to start this every time your server boots up, you can run `systemctl enable`:

```sh
systemctl enable gw.service
```
