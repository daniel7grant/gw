+++
title = "Configuration files"
weight = 4
+++

# Configuration files

Configuration files for services are rarely commited, which means that in case of a fatal issue, you can't restore files, audit changes or rollback. With `gw` you can commit your configurations, and restart the service on change, without ever having to use a VPN or SSH.

## Project configuration

Simply create a new git repository with your specific config files, and push it to a remote server.

## gw configuration

All you have to do is point `gw` to the config directory and reload or restart the service if there are any changes.

```sh
gw /etc/repo -s 'systemctl restart service'
```

## Examples

### nginx configuration

An example configuration could be codifying the reverse proxy. This could be used as a backup for worst-case scenario, but also would help to be able to modify files on your local computer and have those reflected on your production environment.

To start off, create a git repository in your `/etc/nginx` directory, and push it to a remote:

```sh
cd /etc/nginx
git init
git add -A
git commit -m 'Initial commit'
# set a remote and push to it
```

Then you can setup `gw` to reload `nginx` on config change. You can either use `nginx` command or reload using `systemctl`:

```sh
gw /etc/nginx -s 'nginx -s reload'
# or
gw /etc/nginx -s 'systemctl reload nginx'
```

You can also run `nginx` as a subprocess if you don't want to manage it separately, but this will stop and restart it every time a pull occurs:

```sh
gw /etc/nginx -p "nginx -g 'daemon off;'"
```

If you want to avoid getting your system into a bad state by mistake, you can test the config files first with `nginx -t`:

```sh
gw /etc/nginx -s 'nginx -t' -s 'systemctl reload nginx'
```

### DNS configuration with bind

Another great experiment is codifying the most popular DNS service: bind (Berkeley Internet Name Domain). A nice feature of bind is that it can be configured entirely from plaintext files. This means that we can commit our DNS records and modify it with a simple text editor locally. It can also be rolled out to multiple hosts at the same time, thus avoiding any kind of zone transfer. We are actually using a setup like this in production!

To start off, just initialize a git repository in the `/etc/bind` directory and push it to a remote:

```sh
cd /etc/bind
git init
git add -A
git commit -m 'Initial commit'
# set a remote and push to it
```

Then if there are any changes, restart bind with `rdnc` or `systemctl`:

```sh
gw /etc/bind -s 'rdnc reload'
# or
gw /etc/bind -s 'systemctl restart named'
```
