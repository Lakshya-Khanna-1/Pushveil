# What gets scanned

Pushveil models the content introduced by a push rather than assuming `HEAD` is the only relevant commit.

## Existing remote branch

For a normal update, Git supplies the remote's current object ID. Pushveil scans commits reachable from the local object but not from that remote object. Force pushes use the same graph rule, so divergent local commits are still inspected.

## New remote branch

When the remote object ID is all zeros, Pushveil excludes commits already reachable from tracking refs for that remote. If no tracking refs exist, it scans the full history reachable from the local object. This conservative initial-push behavior prevents old credentials from entering a new hosting platform unnoticed.

## Multiple refs

A single push can update several branches or tags. Pushveil processes every update line and combines the resulting commit sets, deduplicating commits and blobs before content scanning.

Deleting a remote ref has an all-zero local object ID and introduces no content, so there is nothing to scan.

## Intermediate commits

Every commit in the introduced range is examined with `git diff-tree`. New object IDs from added or changed paths are scanned, even if later commits:

- delete the file;
- rename the file;
- overwrite the value;
- move the file into another directory;
- merge the branch.

This is why “I deleted the `.env` file in the next commit” does not make the history safe.

## Objects already on the destination

For an existing remote update, objects reachable only from the remote side are excluded. On a new branch, tracking refs for that same remote are used to avoid treating content already present on the destination as newly exposed.

## Working tree and index

The pre-push hook scans committed Git objects. Uncommitted working-tree edits and staged-but-uncommitted changes are not part of a push and are therefore not scanned by the hook. Commit them before testing the push, or use another pre-commit tool for earlier feedback.

## Manual revision scans

`pushveil scan <revision>` passes the supplied revision expression to `git rev-list`. Examples:

```bash
pushveil scan HEAD
pushveil scan -- --all
pushveil scan origin/main..HEAD
```

Use normal Git revision syntax. An invalid or unavailable object causes a fail-closed error.

## Unusual refs

The scanner is optimized for refs resolving to commit history, which covers ordinary branches and annotated commit tags. A tag that directly references a non-commit object is outside the documented push-history model and should be protected with server-side scanning as well.
