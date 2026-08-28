# Pushveil documentation

Pushveil is a local-first Git security tool that scans every commit about to be pushed and blocks credentials before they leave a developer's computer. It runs as a global `pre-push` hook, works with any Git hosting platform, and requires no daemon, account, network service, or per-repository installation.

This book explains how to install and operate Pushveil, how its scanner handles Git history and unusual objects, how to deploy it across an organization, and how to contribute safely to the project.

## Start here

- New user: [Install Pushveil](getting-started/installation.md), then follow the [quick start](getting-started/quick-start.md).
- Push was blocked: read [Understanding a blocked push](getting-started/blocked-push.md).
- Using an AI coding agent: read [AI coding agents](guides/ai-agents.md).
- Configuring rules and allowlists: read [Configuration](configuration.md) and [configuration recipes](guides/configuration-recipes.md).
- Deploying to a team: read [Enterprise deployment](operations/enterprise.md).
- Evaluating security guarantees: read [Security model](security.md).
- Contributing code: read [Development guide](development.md) and [Testing](testing.md).

## The safety promise

For a normal system `git push`, Pushveil determines which commits and blobs would become reachable on the destination remote, streams their contents through its detection engine, and returns a nonzero hook status when it finds a potential secret. Git cancels the push before transferring the refs.

Pushveil scans intermediate commits, so adding a key in one commit and deleting or renaming it in a later commit does not hide it. It also scans raw binary bytes and locally available Git LFS objects without imposing a file-size cutoff.

## What Pushveil does not claim

Pushveil is a developer guardrail, not an unbypassable server policy. Git allows hooks to be skipped with `git push --no-verify`, direct hosting-provider APIs do not invoke local hooks, and remote environments require their own installation. Organizations should combine Pushveil with server-side secret scanning or pre-receive enforcement.

## Project links

- [Source repository](https://github.com/Lakshya-Khanna-1/Pushveil)
- [Issue tracker](https://github.com/Lakshya-Khanna-1/Pushveil/issues)
- [Security reporting](https://github.com/Lakshya-Khanna-1/Pushveil/security/advisories/new)
- [Changelog](https://github.com/Lakshya-Khanna-1/Pushveil/blob/main/CHANGELOG.md)
- [MIT license](https://github.com/Lakshya-Khanna-1/Pushveil/blob/main/LICENSE)
