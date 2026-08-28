# Configuration recipes

Pushveil's defaults require no configuration. Add exceptions only after confirming a finding is not a real credential.

## Repository configuration

Create `.pushveil.toml` in the worktree root:

```toml
[scan]
lfs = true
fail_closed = true
max_findings_shown = 100

[allowlist]
paths = []
rules = []
```

Because configuration uses `deny_unknown_fields`, misspelled keys fail rather than being silently ignored.

## User configuration

Place `config.toml` in the Pushveil application-config directory. See [Configuration](../configuration.md) for platform paths.

When a repository file exists, its scan and allowlist sections replace those user sections. Custom repository rules are appended to custom user rules. Keep organization-wide enforcement outside developer-writable repository configuration when policy must be mandatory.

## Allow a fixture directory

```toml
[allowlist]
paths = ["tests/fixtures/**"]
rules = []
```

Paths are matched as Git-style forward-slash paths. Prefer the narrowest directory or filename pattern possible.

## Allow one rule

```toml
[allowlist]
paths = []
rules = ["jwt"]
```

This suppresses every finding from that rule in the repository. It is broader than a path or line exception and deserves security review.

## Allow one line

```text
example_api_key = "not-a-real-placeholder" # pushveil:allow
```

The marker is case-insensitive and suppresses matches on the same line. It should remain visible in code review.

## Add an internal token format

```toml
[[rules]]
id = "internal-deploy-token"
description = "Internal deployment token"
regex = "deploy_prod_[A-F0-9]{32}"
```

Test both true and false examples with `pushveil scan` before rolling out a custom rule broadly.

## Change terminal verbosity

```toml
[scan]
lfs = true
fail_closed = true
max_findings_shown = 25
```

Pushveil continues scanning after 25 findings. Only terminal display is capped; JSON output and the total count still reflect the result set retained by the scanner.

## Continue after unavailable LFS content

```toml
[scan]
lfs = true
fail_closed = false
max_findings_shown = 100
```

This permits a push when an LFS object could not be verified, provided no secret finding exists. It weakens the security guarantee and is not recommended for normal development or enterprise policy.

## Disable LFS resolution

```toml
[scan]
lfs = false
fail_closed = true
max_findings_shown = 100
```

LFS pointer files remain ordinary small blobs, but their referenced media is not inspected. Use this only when another mandatory control scans LFS content.

