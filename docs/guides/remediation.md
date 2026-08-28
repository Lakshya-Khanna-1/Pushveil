# Removing an exposed secret safely

A blocked push prevents the remote ref update, but the correct response depends on where the credential exists and whether it has ever left the machine.

## First: revoke or rotate

If the value is real, assume compromise whenever it may have appeared in logs, messages, backups, CI artifacts, earlier pushes, or another clone. Revoke or rotate it at the provider before spending time rewriting Git history.

Do not paste the credential into an issue, commit message, terminal transcript, or support request.

## Secret is only in the latest unpushed commit

1. Replace the value with an environment-variable lookup or secret-manager reference.
2. Ensure the secret-bearing file is ignored where appropriate.
3. Amend the commit:

```bash
git add -A
git commit --amend --no-edit
git push
```

Amending replaces the latest commit; merely adding a correction commit leaves the earlier secret-bearing blob in the pushed history.

## Secret is in several unpushed commits

Use an interactive rebase or a dedicated history-rewriting tool to edit or remove every affected commit. Make a backup ref or clone first, inspect the resulting history, and run:

```bash
pushveil scan origin/main..HEAD
```

History rewriting changes commit IDs. Coordinate before rewriting branches shared with other developers.

## Secret has already been pushed

1. Revoke or rotate it immediately.
2. Check provider access logs and incident-response requirements.
3. Enable or confirm server-side secret scanning.
4. Follow the hosting provider's documented history-removal procedure.
5. Coordinate a force push and fresh clones with every collaborator.
6. Remember that forks, caches, pull-request refs, releases, logs, and existing clones may retain the old object.

History removal reduces accidental discovery; it cannot prove every copy has disappeared.

## Store secrets correctly

Prefer:

- a managed secret store;
- environment variables injected outside Git;
- encrypted deployment configuration with keys stored separately;
- short-lived scoped credentials;
- `.env` files excluded by `.gitignore` and generated from documented examples.

Commit `.env.example` with obvious placeholders, not realistic credential-shaped values.

## Confirm the result

Run both a targeted range scan and the intended push:

```bash
pushveil scan origin/main..HEAD
git push
```

If Pushveil still identifies the old path and commit, the secret remains in reachable history. Use that commit ID to continue the rewrite rather than adding another deletion commit.
