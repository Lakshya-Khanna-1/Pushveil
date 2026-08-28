# Glossary

## Blob

A Git object containing file bytes. Paths and filenames belong to tree objects; the same blob can appear at several paths or commits.

## Commit reachability

The set of commits and objects obtainable by walking from a ref. Pushveil reasons about what becomes reachable on the destination after a push.

## Credential

A secret value used to authenticate or authorize access, including API keys, tokens, passwords, and private keys.

## Fail closed

Cancel the operation when security verification cannot complete. Pushveil uses this behavior by default for missing LFS content and core scanner errors.

## Finding

A rule match with classification, path, commit, object ID, source type, and line or byte offset. The matched secret bytes are not included.

## Git hook

An executable invoked by Git at a lifecycle event. Pushveil uses `pre-push` and globally routes other hooks so existing behavior is preserved.

## Git LFS

Git Large File Storage. Commits contain pointer blobs while file content lives in a separate local object store and remote LFS service.

## Gitlink

A tree entry with mode `160000` that records a submodule commit ID rather than file contents.

## Object ID

The hash identifying a Git object. Pushveil deduplicates normal blobs by object ID and resolves LFS content by its SHA-256 ID.

## Pre-push

The client-side Git hook invoked after remote negotiation but before refs are updated. Git supplies local and remote ref/object pairs on standard input.

## Push range

The commits Pushveil determines will be introduced by one or more ref updates. The range differs for existing branches, new branches, force pushes, and remote deletions.

## Remote tracking ref

A local ref such as `refs/remotes/origin/main` representing a known remote branch. Pushveil may use these refs to avoid rescanning history already present when creating a new remote branch.

## Rule ID

A stable machine-readable detector name such as `github-token` or `private-key`. Configuration allowlists refer to these IDs.

## Secret rotation

Invalidating a credential and issuing a replacement. Rotation is the first response when a real credential may have been exposed.

## Server-side enforcement

A policy at the Git host or repository server, such as push protection or a pre-receive hook. It remains effective even when a client-side hook is absent or bypassed.

## Working tree

The checked-out files a developer edits. The pre-push scanner examines committed Git objects, not uncommitted working-tree changes.
