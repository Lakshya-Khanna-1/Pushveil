# Contributing

Contributions are welcome, particularly new high-confidence credential formats, false-positive reductions, performance improvements, and cross-platform tests.

## Setup

Install Rust 1.85 or newer and Git 2.x, then run:

```bash
cargo test --all-targets --locked
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Do not run `pushveil install` from a development build unless you intend to change your user-level Git configuration. Integration tests isolate `GIT_CONFIG_GLOBAL` and `PUSHVEIL_HOME` automatically.

## Detection rules

- Use synthetic credentials in fixtures and tests.
- Prefer a documented provider prefix and fixed structure over broad entropy matching.
- Add a positive test, a near-miss negative test, and a chunk-boundary test when relevant.
- Never paste a credential from an incident report, even after revocation.
- Explain expected false positives and provider documentation in the pull request.

## Pull requests

Keep changes focused, update user documentation for behavior changes, add regression tests, and ensure CI is green. Security-sensitive behavior should fail closed unless the configuration explicitly says otherwise.
