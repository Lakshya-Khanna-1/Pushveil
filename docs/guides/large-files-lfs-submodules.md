# Large files, binaries, Git LFS, and submodules

Pushveil avoids extension-based or size-based skipping. Its behavior depends on how Git stores each object.

## Large normal blobs

Normal Git blobs are streamed through `git cat-file --batch` in bounded chunks. The full blob is read even when it is larger than available memory. Scan time remains proportional to the number of bytes introduced by the push.

`max_findings_shown` limits reporting, not bytes scanned.

## Binary blobs

Pushveil searches raw bytes. If the initial prefix contains a null byte, findings are reported with byte offsets instead of line numbers.

This finds plaintext keys embedded in compiled assets, database dumps, images with metadata, and other binary containers. It does not automatically decrypt content or recursively decompress every archive format. Credentials that exist only in compressed, encrypted, obfuscated, encoded-across-chunks, or generated form may not be visible to raw pattern matching.

## Git LFS

A commit using Git LFS contains a small pointer blob rather than the large file itself. Pushveil recognizes the standard LFS pointer header and SHA-256 object ID, asks `git lfs env` for `LocalMediaDir`, and opens the corresponding local object path.

The LFS object is streamed through the same detector and reported against the repository path. Pushveil runs before the existing LFS `pre-push` hook, so the object should normally still be available locally.

If Git LFS is unavailable or the object is missing, Pushveil records a verification error. With the default `fail_closed = true`, that error blocks the push.

## LFS operational checks

```bash
git lfs version
git lfs env
git lfs status
```

Fetch missing objects before pushing:

```bash
git lfs fetch --all
git lfs checkout
```

Review the storage and network impact before fetching all objects in a very large repository.

## Submodules

A superproject stores a submodule as mode `160000` plus a commit ID. It does not contain the submodule's file blobs, so Pushveil counts the gitlink but cannot scan content absent from the superproject object database.

Install Pushveil in the same user environment and push the submodule repository normally. Its own pre-push invocation scans the submodule commits. Server-side protection should verify that referenced submodule commits came from protected repositories when this is an organizational requirement.

## Monorepos

Monorepos need no special mode. Paths are matched from the repository root, object IDs are deduplicated across packages, and content is streamed. Use path-specific allowlists carefully because a broad monorepo glob can suppress findings across many teams.

## Performance expectations

Initial pushes and rewritten histories may introduce a large object set and therefore take longer than small incremental pushes. This is expected: Pushveil refuses to trade security for an implicit size cutoff. Use manual scans before a time-sensitive release to warm Git object access and discover issues early.

