# Configuration

Pushveil works with secure defaults and no configuration file.

## Locations and precedence

Configuration is loaded in this order:

1. Built-in defaults and detection rules.
2. User configuration at the platform application-config directory:
   - Windows: `%APPDATA%\pushveil\config.toml`
   - macOS: `~/Library/Application Support/pushveil/config.toml`
   - Linux: `${XDG_CONFIG_HOME:-~/.config}/pushveil/config.toml`
3. Repository configuration at `<worktree>/.pushveil.toml`.

Repository scan and allowlist sections replace the corresponding user sections. Custom repository rules are appended to user rules.

`PUSHVEIL_HOME` can override the application-config directory for portable or managed installations.

## Scan behavior

```toml
[scan]
lfs = true
fail_closed = true
max_findings_shown = 100
```

- `lfs`: resolve and inspect Git LFS content referenced by pointer files.
- `fail_closed`: block when some content cannot be verified, such as a missing LFS object.
- `max_findings_shown`: limit terminal output, not scanning. Every byte is still inspected and the total finding count is retained.

## Allowlists

```toml
[allowlist]
paths = [
  "tests/fixtures/**",
  "docs/generated/*.json",
]
rules = ["jwt"]
```

Paths use Git-style forward slashes and glob syntax. Rule IDs are the IDs printed beside findings. Allowlists are security-sensitive and should receive the same review as authentication code.

For one known false positive:

```text
example_api_key = "not-a-real-placeholder" # pushveil:allow
```

Never add the marker beside a live credential.

## Custom rules

```toml
[[rules]]
id = "acme-production-token"
description = "ACME production API token"
regex = "acme_live_[A-Za-z0-9]{48}"
```

Custom expressions use Rust's `regex` syntax over raw bytes. Look-around and backreferences are not supported. Keep expressions bounded where possible; tokens longer than 8 KiB are outside the scanner's cross-chunk matching window.

Invalid configuration or regular expressions fail the push closed.

## Built-in rule IDs

- `aws-access-key`
- `github-token`
- `gitlab-token`
- `openai-key`
- `anthropic-key`
- `google-api-key`
- `stripe-secret-key`
- `slack-token`
- `npm-token`
- `pypi-token`
- `sendgrid-key`
- `twilio-key`
- `private-key`
- `jwt`
- `credential-url`
- `generic-secret`
