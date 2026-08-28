# Development guide

Pushveil is a Rust 2024-edition command-line application with a minimum supported Rust version of 1.85.

## Repository layout

| Path | Responsibility |
|---|---|
| `src/cli.rs` | Public commands, arguments, manual scan entry point, exit codes |
| `src/config.rs` | TOML loading, defaults, paths, and allowlist glob compilation |
| `src/detector.rs` | Streaming byte scanner, rules, finding metadata, placeholders |
| `src/git.rs` | Revision discovery, changed blobs, `cat-file` streaming, LFS resolution |
| `src/hook.rs` | Pre-push orchestration, interactive confirmation, existing-hook chaining |
| `src/install.rs` | User installation, wrappers, state, doctor, and uninstall |
| `src/report.rs` | Human-readable masked terminal output |
| `tests/pre_push.rs` | Isolated end-to-end Git push scenarios |

## Local setup

```bash
git clone https://github.com/Lakshya-Khanna-1/Pushveil.git
cd Pushveil
cargo build --locked
cargo test --all-targets --locked
```

Do not run `pushveil install` from a development build unless you intend to alter your real global Git configuration. Integration tests isolate both `GIT_CONFIG_GLOBAL` and `PUSHVEIL_HOME`.

## Quality commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

The crate forbids unsafe Rust and enables strict Clippy lint groups.

## Adding a built-in rule

1. Obtain authoritative provider documentation for the credential structure.
2. Add a narrow byte regex and stable rule ID in `builtin_rules`.
3. Construct synthetic test values from fragments so the repository does not trigger its own scanner.
4. Add positive, negative, placeholder, and chunk-boundary coverage as appropriate.
5. Run the repository's release binary against the resulting commit.
6. Document the rule ID and expected false positives.

Do not commit a real token, even if revoked.

## Changing Git behavior

Changes to revision ranges, raw diff parsing, hook input, LFS resolution, installation state, or hook chaining require end-to-end tests. Consider new branches, existing branches, force pushes, multiple refs, deleted refs, bare repositories, linked worktrees, prior global hooks, repository hooks, Windows shebang execution, and non-interactive callers.

## Error philosophy

Security verification should fail closed unless a documented configuration setting explicitly permits continuation. Errors must explain the failed boundary without printing secret bytes or constructed shell commands containing untrusted content.

## Pull requests

Keep commits focused, update documentation alongside behavior, and include regression tests. Run the full local quality suite before pushing. See the repository-level `CONTRIBUTING.md` and `SECURITY.md` for community and disclosure expectations.

