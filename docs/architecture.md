# Architecture

## Push lifecycle

1. Git invokes the global `pre-push` wrapper and supplies ref updates on standard input.
2. Pushveil computes each revision set relative to the remote object or the remote's tracking refs.
3. For every commit in those sets, `git diff-tree` identifies added or changed blobs. This preserves evidence from intermediate commits even when a later commit renames or deletes the file.
4. Blob IDs are deduplicated and streamed through one `git cat-file --batch` process.
5. The detector applies byte-oriented rules in 64 KiB chunks with an 8 KiB overlap. Findings retain file, historical commit, line or byte offset, rule, and object provenance—but never the matched value.
6. LFS pointers are resolved from the `LocalMediaDir` reported by `git lfs env`; their local objects are streamed through the same detector.
7. A clean scan invokes the hook configuration that existed before installation. A blocked scan requires an exact interactive override or exits nonzero.

## Global hook routing

Git supports one `core.hooksPath`. The installer records its prior value, writes small wrappers for every standard Git hook name, and then activates its own directory globally. Only `pre-push` performs a security scan; all other wrappers dispatch immediately to the previous global hook directory or the repository's physical `.git/hooks` directory. Uninstall restores the recorded setting only if Git still points to this installation.

## Failure policy

Parsing errors, invalid rules, Git object failures, and other core verification failures return a nonzero hook status. Missing LFS content is recorded as a verification error and blocks when `scan.fail_closed` is true. Non-interactive sessions always choose cancellation rather than attempting to synthesize an override.

## Trust boundaries

The program trusts the local Git executable and object database. It does not trust repository content, paths, configuration syntax, or hook metadata. Hook names are validated against path traversal, commands are invoked without a shell except when Git for Windows must execute an existing shebang hook, and detected values are excluded from output.
