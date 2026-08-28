# Security policy

## Supported versions

Security fixes are provided for the latest released version.

## Reporting a vulnerability

Do not open a public issue for a vulnerability that could expose credentials or bypass scanning. Use GitHub's **Security → Report a vulnerability** private-reporting flow after this repository is published. Include the affected version, operating system, Git version, reproduction steps, and impact. Never include a real credential; use a clearly revoked or synthetic test value.

Maintainers should acknowledge a report within five business days, validate and classify it, prepare a coordinated fix, and publish an advisory when users can update safely.

## If a credential was committed

Treat it as compromised even if a push was blocked: terminal history, local logs, backups, or other tools may have copied it. Revoke or rotate the credential first. Then remove it from the working tree and rewrite Git history where appropriate.
