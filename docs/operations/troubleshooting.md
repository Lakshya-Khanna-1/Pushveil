# Troubleshooting

Start with:

```bash
pushveil doctor
git --version
git config --global --get core.hooksPath
```

## `Pushveil is not installed`

Run the downloaded or locally built executable with `install`. If `pushveil` is not on `PATH`, use its full path:

```powershell
.\target\release\pushveil.exe install
```

The hook uses an absolute path to the installed versioned binary, so daily pushes do not require the original download directory to remain on `PATH`.

## Pushes run without a scan message

Check these conditions:

1. The push uses the system Git executable.
2. It does not include `--no-verify`.
3. The active user and environment match the installation.
4. `core.hooksPath` points to the directory reported by `doctor`.
5. The `pre-push` wrapper and installed executable still exist.

Re-run `pushveil install` to refresh a damaged wrapper while preserving the original prior hook path.

## Push fails with no interactive terminal

This is expected when findings or fail-closed errors occur in an agent, CI job, IDE capture, or redirected shell. Read the findings above the message and remediate them. Pushveil does not accept a non-interactive override.

## Existing hook stopped running

Run `doctor`, then inspect `install-state.toml` in the Pushveil application-config directory. It records `previous_hooks_path`. When no global hook path existed before installation, Pushveil dispatches to the repository's physical `.git/hooks/<hook-name>`.

Do not manually point `previous_hooks_path` back to Pushveil's own directory; that can create recursion. Reinstall or uninstall and reinstall if state is damaged.

On Unix, an existing local hook must have an executable permission bit, matching Git's normal behavior.

## Invalid configuration

Pushveil fails closed and prints the path plus TOML parsing error. Check:

- unknown field names;
- missing quotes or brackets;
- invalid glob syntax;
- duplicate TOML tables;
- custom regex syntax unsupported by Rust `regex`.

Temporarily move the configuration outside the repository only for diagnosis, never as a way to bypass a real finding.

## LFS object cannot be resolved

Confirm Git LFS is installed and the object exists locally:

```bash
git lfs version
git lfs env
git lfs status
git lfs fetch
git lfs checkout
```

The `LocalMediaDir` from `git lfs env` must contain the SHA-256 object referenced by the committed pointer.

## Scan is unexpectedly slow

Initial branches, monorepos, force pushes, and rewritten histories may introduce many bytes. Use:

```bash
git rev-list --count origin/main..HEAD
git count-objects -vH
```

Pushveil intentionally has no silent file-size cutoff. Reduce the actual push range rather than disabling scanning.

## False positive repeats after correction

Read the reported commit. If it is older than the latest commit, the original blob remains in history. Amend or rewrite that commit; a later deletion does not remove it.

## Uninstall refuses to change hooks

Pushveil refuses when the current `core.hooksPath` no longer matches its recorded installation. This protects a newer user configuration. Decide which hook system should own the setting, then change it explicitly rather than forcing Pushveil to overwrite it.

## Collecting a safe bug report

Include the Pushveil version, Git version, operating system, sanitized command, rule ID, file type, and whether the push was interactive. Never include the matched credential or a real secret-shaped reproduction value.

