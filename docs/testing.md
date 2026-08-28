# Testing and verification

Pushveil uses unit tests for detector and parsing boundaries plus integration tests that create real isolated repositories and bare remotes.

## Unit coverage

Current unit tests verify:

- a provider key split across streaming chunks is detected;
- plaintext secrets inside binary data are detected and use byte offsets;
- the inline `pushveil:allow` marker suppresses its line;
- Git LFS pointer parsing validates a SHA-256 object ID;
- pre-push input lines are parsed correctly;
- only the exact uppercase `PUSH ANYWAY` phrase is accepted.

## End-to-end coverage

Integration tests build temporary Git repositories and exercise the installed global hook path while isolating user configuration. They verify:

- a clean push succeeds;
- an existing repository `pre-push` hook still runs;
- a non-interactive agent-style push is blocked;
- a secret in an earlier commit remains detected after deletion;
- a secret remains detected after a later rename;
- raw binary blobs are scanned;
- a local Git LFS object is resolved and scanned.

Tests use local bare remotes and never contact GitHub or alter the developer's real global configuration.

## Run tests

```bash
cargo test --all-targets --locked
```

Show captured output for one test:

```bash
cargo test --test pre_push non_interactive_agent_push -- --nocapture
```

The filter can be abbreviated as long as it selects the intended test.

## Release verification

Before publishing a tag:

1. Run formatting, Clippy, tests, and the optimized build.
2. Run `target/release/pushveil scan HEAD` on the release commit.
3. Confirm a clean working tree and matching version/changelog.
4. Let GitHub CI pass on Linux, macOS, and Windows.
5. Verify generated archive checksums.
6. Smoke-test install, clean push, blocked push, chained hook, doctor, and uninstall on each operating system.
7. Sign and notarize artifacts where applicable.

## Manual interactive override test

Use an isolated synthetic repository with no real credentials. Trigger a known test finding, run `git push` from a genuine terminal, verify Enter cancels, then repeat and type `PUSH ANYWAY`. Confirm that the override applies once and any chained hook still runs.

Never perform this test against a production remote or with a live key.

## Adding regression coverage

A security bug should receive a test that fails before the fix and passes after it. Keep synthetic secret strings fragmented in source code so Pushveil can safely scan its own repository history.

