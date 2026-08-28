# Quick start

After installation, use Git exactly as before. Pushveil has no background process and needs no per-repository initialization.

## Confirm protection

From the same terminal and user account that will push:

```bash
pushveil doctor
git config --global --get core.hooksPath
```

The first command should show all checks as `[ok]`. The second should print Pushveil's hook directory.

## Make a normal push

```bash
git add .
git commit -m "Add payment integration"
git push
```

A clean push prints a summary similar to:

```text
✓ scanned 2 commit(s), 14 blob(s), 38.2 KiB — no secrets found
```

Git then continues the push and runs any existing `pre-push` hook.

## Run a manual scan

Scan every commit reachable from `HEAD`:

```bash
pushveil scan HEAD
```

Scan only a feature branch's commits relative to its base:

```bash
pushveil scan origin/main..HEAD
```

Produce JSON for scripts or CI:

```bash
pushveil scan origin/main..HEAD --json
```

Manual scanning is useful before opening a pull request, when reviewing imported history, or when a remote push is not yet configured.

## Interpret exit status

- `0`: scan completed and no blocking condition was found.
- `1`: findings exist, fail-closed verification errors occurred, installation health is bad, or the hook cancelled the push.

Git uses the nonzero hook status to stop the push.

## Next steps

- Learn what happens when a finding appears in [Understanding a blocked push](blocked-push.md).
- Review repository settings in [Configuration](../configuration.md).
- Understand historical coverage in [What gets scanned](../concepts/push-coverage.md).

