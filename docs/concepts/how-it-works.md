# How Pushveil works

Pushveil uses Git's native hook lifecycle. It does not replace `git push`, proxy network traffic, or require knowledge of the hosting provider.

## Pre-push sequence

1. The user or coding agent runs the system `git push` command.
2. Git negotiates with the remote and determines which local and remote refs are involved.
3. Before transferring the update, Git invokes Pushveil's global `pre-push` wrapper.
4. Git sends lines containing local ref, local object ID, remote ref, and remote object ID to the hook's standard input.
5. Pushveil computes the commits that would become newly reachable on that remote.
6. It identifies blobs added or changed by every commit in the range and deduplicates identical object IDs.
7. One long-lived `git cat-file --batch` process streams those blobs into the detector.
8. Git LFS pointer blobs are resolved to locally stored LFS objects and streamed through the same detector.
9. A clean scan passes the original hook input to the previously configured `pre-push` hook and returns its status.
10. Findings or fail-closed verification errors return a nonzero status unless an interactive user explicitly overrides them.

Git cancels a push whenever a `pre-push` hook returns nonzero.

## Why Pushveil scans Git objects

Scanning only the working directory or the latest commit misses important cases. A secret can be added, committed, renamed, and deleted while remaining permanently reachable through an intermediate commit. Pushveil asks Git for the actual history and object IDs involved in the push.

Object-level deduplication also avoids rescanning identical content that appears under multiple paths or across several commits.

## Streaming and memory use

The detector reads 64 KiB chunks with an 8 KiB overlap. The overlap lets bounded credential patterns cross chunk boundaries. The scanner retains a small prefix for binary and Git LFS identification, plus finding metadata, rather than loading each file or repository into memory.

The terminal output limit affects only how many findings are displayed. It does not stop scanning and does not change the total finding count.

## Hook coexistence

Git accepts one global `core.hooksPath`. Pushveil records the previous value and installs small dispatch wrappers for standard hook names. Only `pre-push` runs the scanner. Every other wrapper forwards directly to the prior global hook directory, or to the repository's physical hook when no global path existed.

For `pre-push`, Pushveil preserves Git's original arguments, standard-input ref updates, standard output, standard error, and exit status when invoking the chained hook.

## Network and privacy

Pushveil makes no service call during scanning. It reads the local Git object database, local configuration, and local Git LFS storage. Findings exclude the matched value, and the program does not create an application log.

Git itself still communicates with the configured remote as part of the surrounding push operation.

