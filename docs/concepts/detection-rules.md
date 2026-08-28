# Detection engine and rules

Pushveil combines high-confidence provider formats with contextual credential patterns. Rules operate on raw bytes, so they work in UTF-8 text and in binary content containing plaintext credentials.

## Built-in rules

| Rule ID | Detects |
|---|---|
| `aws-access-key` | AWS access key IDs beginning with `AKIA` or `ASIA` |
| `github-token` | GitHub classic, fine-grained, OAuth, user, server, and refresh token formats |
| `gitlab-token` | GitLab personal/project/group access-token format |
| `openai-key` | OpenAI project, service-account, and legacy API-key formats |
| `anthropic-key` | Anthropic API keys |
| `google-api-key` | Google API keys beginning with `AIza` |
| `stripe-secret-key` | Stripe secret and restricted keys |
| `slack-token` | Slack token families beginning with `xox` |
| `npm-token` | npm granular access tokens |
| `pypi-token` | PyPI upload tokens |
| `sendgrid-key` | SendGrid API keys |
| `twilio-key` | Twilio API keys |
| `private-key` | Common PEM, OpenSSH, and PGP private-key headers |
| `jwt` | Three-segment JSON Web Tokens |
| `credential-url` | Credentials embedded in HTTP or common database URLs |
| `generic-secret` | Long values assigned to sensitive names such as `api_key`, `access_token`, `client_secret`, `aws_secret_access_key`, or `password` |

Provider rules run before the generic contextual rule. When a provider-specific finding and generic assignment occur on the same line, Pushveil suppresses the redundant generic result.

## Finding locations

Text without a null byte in its prefix receives a one-based line number. Binary content receives an absolute byte offset. A finding also records the object ID, historical commit, path, and whether the source was a normal blob or Git LFS object.

## Placeholder handling

Common explicit placeholder markers such as `replace_me`, `your_api_key`, `not-a-real`, `redacted`, and `changeme` are ignored to reduce documentation and template noise. Synthetic examples should use unmistakable placeholder language instead of values shaped exactly like live credentials.

## Custom rules

Add byte-oriented Rust regular expressions in configuration:

```toml
[[rules]]
id = "acme-production-token"
description = "ACME production API token"
regex = "acme_live_[A-Za-z0-9]{48}"
```

Custom expressions do not support look-around or backreferences. Keep tokens bounded and shorter than the 8 KiB cross-chunk overlap.

## Detection limits

No pattern engine can identify every secret. A random value without a recognizable provider format or sensitive context may look identical to normal data. Raw scanning also cannot reveal plaintext that exists only after decryption, decompression, decoding across multiple files, or runtime generation.

Rules should favor high confidence over maximum recall. Excessive false positives train developers to override real warnings.

## Reporting a missed format

Open a feature request with provider documentation and fully synthetic examples. Never paste a live or incident-derived credential into an issue, test, commit, or pull request.
