# Frequently asked questions

## Does Pushveil work with GitHub, GitLab, and Bitbucket?

Yes. It runs before the transport-specific push completes, so normal system Git pushes work with any remote platform or local bare repository.

## Does it upload source code for scanning?

No. Scanning uses the local Git object database and local Git LFS storage. Pushveil has no service or telemetry endpoint.

## Does it scan every repository automatically?

It applies to pushes made by the installed operating-system user and Git environment. No per-repository setup is needed. Separate WSL distributions, containers, VMs, remote hosts, and user accounts need separate installation.

## Does it scan before commit?

No. The security gate is `pre-push`. This avoids blocking normal commits and examines the exact history being sent. A separate pre-commit scanner can provide earlier feedback.

## Can it find a secret that was deleted later?

Yes, when the earlier secret-bearing commit is part of the introduced push range. Pushveil scans intermediate commits rather than only the latest tree.

## Does it scan large and binary files?

Yes. Normal blobs are fully streamed with bounded memory and raw binary bytes are searched. There is no file-size skip. Encrypted or compressed-only plaintext may not be visible.

## Does it scan Git LFS files?

Yes, when Git LFS is installed and the referenced object exists locally. Missing LFS data blocks by default.

## Does it scan submodule contents?

Not from the superproject, which contains only a submodule commit ID. The submodule repository is scanned when its own commits are pushed from a protected environment.

## Can an AI agent use it?

Yes. An agent running normal system Git receives the masked report and a blocked exit status. Non-interactive agents cannot override a finding.

## Can a developer bypass it?

Yes. Git exposes `--no-verify`, and a user controlling the workstation can alter hooks or configuration. Use server-side protection for mandatory enforcement.

## Why not print part of the secret?

Even partial values can leak sensitive identifiers into terminal history, agent transcripts, CI logs, or screenshots. Location and rule metadata are sufficient for investigation.

## Why did deleting the file not fix the push?

The earlier commit still references its blob. Amend or rewrite the affected unpushed history rather than adding another deletion commit.

## How do I report a false positive?

Provide the rule ID, sanitized structure, operating system, and minimal synthetic reproduction. Never include a live or incident-derived credential.

## Is Pushveil a replacement for GitHub secret scanning?

No. Pushveil provides immediate local feedback. Hosting-provider and server-side scanning cover bypasses and unprotected machines. Use both.

