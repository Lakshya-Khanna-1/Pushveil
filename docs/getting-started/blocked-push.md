# Understanding a blocked push

When Pushveil finds a potential credential, it stops before Git updates the remote refs. Secret values are never echoed to the terminal.

```text
✗ Push blocked: 1 potential secret(s) found

  OpenAI API key  services/payments/.env.production:3
      commit 1af5e9d2d731 · Git blob · rule openai-key

Secret values are intentionally masked from terminal output.

[Enter or type OK] Cancel this push
[Type PUSH ANYWAY] Override once and continue
>
```

## Read a finding

Each finding includes:

- a human-readable secret category;
- the historical file path;
- a line number for text or byte offset for binary content;
- the commit in which the scanned blob appeared;
- whether the source was a Git blob or Git LFS object;
- the rule ID for configuration and investigation.

The full matched credential is intentionally omitted so terminal capture, agent logs, and CI output do not create another copy of the secret.

## Safe response

Press Enter or type `OK` to cancel. Then:

1. Determine whether the finding is a real credential.
2. Revoke or rotate a real credential immediately if it may have been shared anywhere.
3. Remove it from the file and from every unpushed commit that contains it.
4. Commit or amend the correction.
5. Run `git push` again. Pushveil rescans the new range from scratch.

Deleting a file in a later commit is insufficient. Git retains the earlier blob, and Pushveil deliberately scans that history.

## Intentional override

An interactive user can type the exact uppercase phrase:

```text
PUSH ANYWAY
```

The override applies only to that invocation. It is intentionally unavailable to non-interactive processes and should be reserved for a reviewed false positive or a controlled test repository. Existing hooks may still reject the push afterward.

## Non-interactive and AI-agent pushes

When no interactive terminal is attached, Pushveil reports the findings and cancels immediately. It never waits indefinitely and never infers approval from standard input. The calling agent or automation receives the nonzero status and can remediate the files, but a human must perform an intentional override.

## False positives

Prefer the narrowest exception:

1. Improve or replace a custom rule.
2. Add `pushveil:allow` to the specific false-positive line.
3. Allowlist a narrowly scoped fixture path.
4. Disable a built-in rule only when the entire rule is inappropriate for the repository.

Never allowlist a real credential. See [Configuration recipes](../guides/configuration-recipes.md).
