+++
title = "Comparison"
weight = 1
+++

# Comparison

There are a lot of tools that can help you deploy code only by pushing to a git repository. `gw` offers a completely unique proposition without locking yourself into expensive cloud services or complicated setups like Kubernetes.

## ArgoCD

**Similar products**: [FluxCD](https://fluxcd.io/)

The main inspiration for `gw` was undeniably [ArgoCD](https://argo-cd.readthedocs.io/en/stable/). It is a great tool that can access git repositories and sync your application to Kubernetes with the latest changes. Leveraging the Kubernetes platform it can reconcile changes (you don't need to figure out imperatively how to update) and provides autohealing, no-downtime rollouts and simple rollbacks. There is a web interface which allows to visualise, redeploy or rollback the applications with one click.

The main disadvantage of ArgoCD (and similar tools) is the tight integration with Kubernetes. For smaller applications it's not worth to maintain a Kubernetes cluster, but you might want to still use GitOps. If you don't need scalability, it is less complex to setup a `gw` script on a cheap VPS.

## Netlify

**Similar products**: [Vercel](https://vercel.com/), [Cloudflare Workers/Pages](https://workers.cloudflare.com/)

Cloud tools like [Netlify](https://www.netlify.com/) were the first ones that really moved automatic deployments to the mainstream. You can connect a git repository to Netlify, which then builds and deploys a separate version of your application on every commit. You can preview these and promote them to production or rollback if an issue arises. Netlify also takes care of DNS and certificate management.

However with Netlify you can only deploy some compatible stacks (static application with serverless functions). If you want to deploy full-stack applications or need advanced features (e.g. task management or notifications), you might need to pay for separate services... if you can do it at all. These cloud-based vendors also lock you into their services, which makes it harder to move between the platforms. `gw` is entirely platform-independent and can build and deploy your application even if it is completely full-stack. By deploying to a single VPS you can avoid suprising bills that you can't get out of.

## GitHub Actions

**Similar products**: [GitLab CI](https://docs.gitlab.com/ee/ci/), [Jenkins](https://www.jenkins.io/)

A common way to deploy applications automatically is push-based CD. It means using CI (for example [GitHub Actions](https://docs.github.com/en/actions)) to build and push the code to the server. It can be useful because it can integrate with your already existing solutions. You check the code by unit and integration testing before pulling the trigger on a deployment. It also provides pre-built actions which can handle complex use-cases (Docker building, deploying IaaC).

The biggest drawback of push-based deployments is that it needs access to your server. If you just want to copy some code to a server it might be a security risk to allow SSH access from an untrusted CI worker. It can get even more complicated when your servers are in a secure network (behind NAT or VPN). On the other hand, `gw` can run on your server and can pull the code, avoiding the security problems altogether.

## Watchtower

[Watchtower](https://containrrr.dev/watchtower/) provides a half-push, half-pull approach using the common element between the CI and the server: the image registry. You can use your existing CI infrastructure to build a Docker image and push it to an image registry, while the server can listen to the registry and update the running containers on demand. This is a very good solution that can use existing CI code, while also providing instant feedback from the server.

The main issue with this solution is that it puts the image registry under a lot of pressure. If you are deploying often, the storage and network costs might climb very quickly. If you are building large Docker images, it can also get considerable slow: CI-s rarely cache effectively and pushing and pulling from a registry can also be a slow operation. With `gw` you can save this roundtrip, building the Docker image right on the server. It also improves caching because the previous version of the image is right there. You have to be careful not to overload your server but it might be worth to avoid slow and expensive registries.

## Coolify

**Similar products**: [Dokploy](https://dokploy.com/)

[Coolify](https://coolify.io/) is very definitely the closest product to `gw`: an open-source, self-hostable, pull-based deployment platform that runs everywhere. You can install Coolify and all of their dependencies with a single command. It has a neat web interface, can manage databases with external backups and provides a reverse proxy with automatically renewing certificates. It is a great solution if you want to have everything set up automatically!

The main problem with Coolify is that it takes over your server entirely. Instead of working with your existing deployments, it handles Docker, reverse proxying and certificates. It might be an excellent way of running things, but if you have some specific feature or you already have a running server, it might be a serious investment to transfer. Compared to this, `gw` draws from the UNIX philosophy: it is a modular piece of software that you can integrate with other parts to achieve what you want. You can slot in `gw` to almost any existing deployment, but expect to handle databases, certificates and other problems with different tools.

## while true; do git pull; done

> I can do all of this with shell scripts...

While it is true that in the core, `gw` is just a loop running git pull and actions, but it is also much more. With shell scripts you have to handle git polling, process management and logging, while also handling errors without crashing the script. Not to mention more advanced features like graceful shutdowns, multiple scripts and processes, git authentication or webhooks. `gw` is a very lightweight binary that provides all of these, while being configurable, portable and reliable. If you prefer to write the shell script you can do it, but if you don't, just drop `gw` in and let it handle the boring parts for you!
