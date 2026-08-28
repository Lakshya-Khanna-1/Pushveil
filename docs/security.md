# Security model

This chapter defines what Pushveil protects, what it trusts, and where another control is required.

## Security objective

For a normal system `git push` executed in an installed user environment, inspect the committed content being introduced to the destination and return a blocking status before the remote ref update when a potential secret or fail-closed verification error is found.

## Protected assets

- API keys and provider tokens recognizable by built-in or custom rules
- private-key headers
- credentials embedded in supported URL patterns
- contextual password and secret assignments
- plaintext credentials in normal blobs, binary blobs, and local Git LFS objects

## Trust assumptions

Pushveil trusts:

- the local Git executable and object database;
- the operating system to execute the installed binary;
- Git's pre-push metadata;
- the user-level configuration and installation state;
- Git LFS's reported local media directory.

Repository content, paths, TOML syntax, regex configuration, and hook arguments are treated as untrusted input. Hook names are checked for path traversal, commands are passed as argument arrays, and secret matches are excluded from output.

## Guaranteed failure behavior

Core Git failures, invalid configuration, invalid rules, malformed hook input, and unreadable normal Git objects return nonzero. Missing or unavailable LFS content becomes a verification error and blocks by default when `fail_closed = true`.

Non-interactive finding flows always cancel.

## Explicit bypasses

The local model cannot prevent a user or compromised process from:

- running `git push --no-verify`;
- changing `core.hooksPath`;
- replacing the installed binary or wrapper;
- changing repository allowlists;
- pushing with another Git client that ignores hooks;
- using a hosting-provider API;
- pushing from another environment.

These are reasons for server-side enforcement, not defects that a user-controlled local hook can eliminate.

## Detection limitations

Pushveil may miss:

- unknown formats without sensitive context;
- secrets split across files or generated at runtime;
- encoded values that no rule recognizes;
- plaintext existing only after decompression or decryption;
- credentials longer than the cross-chunk matching assumptions;
- content absent from the local Git object database, including submodule file trees stored elsewhere.

It may also produce false positives when real application data resembles a credential.

## Privacy properties

Scanning is local. Pushveil has no account, telemetry endpoint, update service, or application log. Terminal findings contain classification and location metadata but not the matched bytes.

Git network activity still occurs as part of ordinary remote negotiation surrounding the hook.

## Dependency and release security

The repository pins dependency resolution in `Cargo.lock`, runs strict linting and cross-platform tests, and includes a scheduled RustSec audit workflow. GitHub Actions and Rust dependencies remain part of the supply chain and require timely review. Release consumers should verify published hashes and signatures.

## Reporting vulnerabilities

Use [GitHub private vulnerability reporting](https://github.com/Lakshya-Khanna-1/Pushveil/security/advisories/new). Do not open a public proof of concept that exposes a reliable bypass before maintainers can coordinate a fix, and never include a real credential.

