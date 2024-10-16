+++
title = "Netlify alternative"
weight = 7
+++

# Netlify alternative

If you want to generate a version for every commit of your code, and expose it as hashes, you can `gw` and environment variables.

> Note: This solution **won't** be exactly like Netlify, it won't have the web UI, on-click rollbacks, etc. As this already requires a generous amount of ducktape, please be vary of using this on production. If you need all the features of Netlify, just use Netlify.

## Project configuration

The main idea behind this solution is that most static site generators allow changing output directories. We can wire this together with the git short hash set in the environment by `gw`, and build the different versions side-by-side.

For this you have to find the output directory flag in your static site generator and set it to the `GW_GIT_COMMIT_SHORT_SHA` variable. For example for Jekyll and Hugo this is the `--destination` flag, in 11ty this is the `--output` flag.

```sh
jekyll build --destination=output/$GW_GIT_COMMIT_SHORT_SHA
hugo --destination=output/$GW_GIT_COMMIT_SHORT_SHA
npx @11ty/eleventy --input=. --output=output/$GW_GIT_COMMIT_SHORT_SHA
```

## gw configuration

You can use this command to configure your `gw`:

```sh
gw /path/to/repo -s 'jekyll build --destination=output/$GW_GIT_COMMIT_SHORT_SHA'
```

To build another version for the latest you can copy the files to another folder:

```sh
gw /path/to/repo -s 'jekyll build --destination=output/$GW_GIT_COMMIT_SHORT_SHA' -s 'cp -r output/$GW_GIT_COMMIT_SHORT_SHA output/latest'
```

## Web server configuration

One extra setup that you have to do is point your web server to this directory. By default you can use it as path prefixes, but it can be configured to sub domains to these directories. That way you could reach the commit `0c431ff1` on the url `0c431ff1.example.net`.

> Make sure to setup wildcard domains in your DNS server so it redirects all domains to your server!

## Nginx

You can use regexes in `server_name` to rewrite subdomains into different folders. For example this configuration will resolve `0c431ff1.example.net` to `/path/to/repo/0c431ff1`:

```sh
http {
    server {
        # It will capture the subdomain (e.g. 0c431ff1.example.net)
        server_name ~^([0-9a-f])\.example\.net$;

        location / {
            # And resolve to /path/to/repo/0c431ff1
            root /path/to/repo/$1;
        }
    }

    # You can add another to reach the latest
    server {
        server_name example.net;

        location / {
            root /path/to/repo/latest;
        }
    }
}
```