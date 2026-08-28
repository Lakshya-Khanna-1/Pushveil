# Pushveil

Pushveil is a small, local-first Rust CLI that stops API keys and other credentials before a Git push leaves your computer. Install it once and it protects every existing and future repository for your user account, on any Git hosting platform.

## Install and start using it

Download and extract the executable for your operating system from the repository's **Releases** page.

Windows PowerShell:

```powershell
.\pushveil.exe install
.\pushveil.exe doctor
```

macOS or Linux:

```bash
chmod +x ./pushveil
./pushveil install
./pushveil doctor
```

Then use Git normally in any repository—no per-project setup is required:

```bash
git add .
git commit -m "Your change"
git push
```

The `install` command configures protection for every existing and future repository used by the current operating-system user. `doctor` confirms that the binary, pre-push hook, and global Git routing are active.

It has no daemon, account, network service, telemetry, or language runtime. Git starts it only when a hook is needed.

```text
$ git push

✗ Push blocked: 2 potential secret(s) found

  OpenAI API key  services/payments/.env.production:3
      commit 1af5e9d2d731 · Git blob · rule openai-key
  Private key  infrastructure/deploy.pem:1
      commit b8b9b14d836e · Git blob · rule private-key

Secret values are intentionally masked from terminal output.

[Enter or type OK] Cancel this push
[Type PUSH ANYWAY] Override once and continue
> _
```

## Why this project is different

- **Scans the push, not only the latest checkout.** Every commit about to become reachable on the remote is inspected. A secret added in one commit and deleted or renamed in a later commit is still detected.
- **No file-size cutoff.** Content is streamed in bounded chunks, so unusually large blobs do not have to fit in memory.
- **Text and binary content.** Raw bytes are scanned instead of silently skipping files classified as binary.
- **Git LFS aware.** LFS pointer files are resolved and the corresponding local LFS objects are scanned before the existing LFS upload hook runs.
- **Safe override.** A blocked interactive push continues only after the exact phrase `PUSH ANYWAY` is typed. Pressing Enter, typing `OK`, or running without a terminal cancels it.
- **Existing hooks keep working.** The installer chains the previous global hook directory or the repository's own hook rather than discarding it.
- **Platform-independent.** The remote can be GitHub, GitLab, Bitbucket, Azure DevOps, a self-hosted server, or a local bare repository.

## Installation

Git 2.x is the only runtime prerequisite.

### Prebuilt release

Download the archive for your operating system from the repository's **Releases** page, extract it, then run:

Windows PowerShell:

```powershell
.\pushveil.exe install
```

macOS or Linux:

```bash
chmod +x ./pushveil
./pushveil install
```

The installer copies a versioned executable to the user's application-config directory and sets Git's global `core.hooksPath`. Administrator privileges are not required.

### Build from source

Rust 1.85 or newer is required only when building:

```bash
git clone https://github.com/Lakshya-Khanna-1/Pushveil.git
cd Pushveil
cargo build --release --locked
./target/release/pushveil install
```

On Windows, the last command is:

```powershell
.\target\release\pushveil.exe install
```

## Everyday usage

Use Git normally:

```bash
git add .
git commit -m "Add payment integration"
git push
```

For a clean push, Pushveil prints a compact summary and lets Git continue. If it finds a potential credential, it reports the rule, historical commit, file, and line or binary byte offset without printing the credential itself.

Choose one of two actions:

- Press **Enter** or type `OK` to cancel. Correct the file and its Git history as necessary, commit the correction, and push again. The next push is scanned from scratch.
- Type the exact uppercase phrase `PUSH ANYWAY` to allow only the current interactive push. This is deliberately difficult to do by accident.

Non-interactive pushes fail closed and never wait for input.

## AI coding agent compatibility

Pushveil is independent of the coding agent. It protects the Git executable and user environment in which it is installed, so it works with **Codex, Claude Code, Antigravity, Cursor, Windsurf, Cline, Aider, Gemini CLI, and other coding agents** when they run the normal system `git push` command on the protected machine.

Typical agent flow:

```text
AI coding agent
  → runs the system `git push`
  → Git invokes the global pre-push hook
  → Pushveil scans every commit being introduced
  → clean push continues, or a finding exits nonzero and returns the report to the agent
```

The agent receives the same masked file, line, commit, and rule output that a person sees in the terminal. If the agent is running non-interactively, a finding always cancels the push; it cannot answer the `PUSH ANYWAY` prompt or hang while waiting for input. The agent can then remove the credential, commit the correction, and try again. A human must perform any intentional interactive override.

This also protects code written entirely by an agent but pushed later by a person—the scan runs at push time, regardless of who created the commits.

### When installation is required again

Install Pushveil separately inside every environment that has its own Git executable, user configuration, or filesystem. Examples include:

- WSL distributions
- Docker containers and development containers
- virtual machines
- SSH development hosts
- remote or cloud coding-agent sandboxes
- a different operating-system user account

Installing it on Windows does not automatically install it inside WSL or a container. Run the appropriate Pushveil binary and `pushveil install` inside each environment that can push.

### Paths that cannot invoke local Git hooks

A local hook cannot inspect a push when an agent bypasses the protected Git executable, including:

- calling the GitHub/GitLab REST or GraphQL API to create commits or update refs directly;
- using a Git library or embedded client that does not execute native Git hooks;
- running `git push --no-verify`;
- pushing from an unprotected cloud worker or another computer.

For these cases, enable the Git provider's server-side push protection or a self-hosted `pre-receive` hook as the mandatory final boundary. In agent instructions and enterprise policy, require normal system Git pushes and prohibit `--no-verify`.

### Verify an agent environment

Open the exact terminal or sandbox the agent uses and run:

```bash
pushveil doctor
git config --global --get core.hooksPath
```

`doctor` should report that the binary, pre-push hook, and global hook routing are active. If the agent can run shell commands, you can also ask it to run these checks before its first push. No agent-specific plugin or integration is required.

### Manual history scan

Scan every commit reachable from `HEAD`:

```bash
pushveil scan HEAD
```

Scan a revision range:

```bash
pushveil scan origin/main..feature/payment-work
```

Machine-readable output:

```bash
pushveil scan HEAD --json
```

The scan command exits with status `1` when findings exist or when fail-closed verification errors occur.

### Check or remove the installation

```bash
pushveil doctor
pushveil uninstall
```

`uninstall` restores the exact global hook setting that existed before installation. Cached versioned binaries remain in the application-config directory so Windows never has to delete the executable currently performing the uninstall; they are inert after hook routing is restored.

## What is detected

Built-in rules cover common formats and contextual assignments, including:

- AWS access-key IDs and contextual AWS secret-access keys
- GitHub, GitLab, OpenAI, Anthropic, Google, Stripe, Slack, npm, PyPI, SendGrid, and Twilio tokens
- PEM/OpenSSH/PGP private-key headers
- JWTs
- credentials embedded in HTTP and common database URLs
- values assigned to names such as `api_key`, `access_token`, `client_secret`, and `password`

Rules operate locally and can be extended without recompiling. See [Configuration](docs/configuration.md).

## Repository and file handling

| Git content | Behavior |
|---|---|
| Multiple commits | Scans all commits introduced by every ref in the push |
| Deleted files | Scans the earlier commit in which the blob existed |
| Renamed files | Scans the blob and reports its historical path |
| Monorepos | Uses the same streaming, deduplicated object scan; no special setup |
| Large files | Streams the entire blob with bounded memory; no size-based skip |
| Binary files | Scans raw bytes and reports byte offsets |
| Git LFS | Resolves and scans local LFS content; missing content is a fail-closed verification error by default |
| Submodules | Scans the gitlink metadata in the superproject; file contents are scanned by the submodule repository's own push |
| Branch/tag pushes | Computes content newly reachable on the destination remote |
| Force pushes | Scans commits reachable locally but not from the remote's reported old object |

Compressed or encrypted content cannot always reveal its plaintext through a raw-byte scan. See [Security model](#security-model-and-limitations).

## Configuration

The default configuration is intentionally usable without any files. A repository may add `.pushveil.toml`; a user may add `config.toml` under the Pushveil application-config directory.

```toml
[scan]
lfs = true
fail_closed = true
max_findings_shown = 100

[allowlist]
paths = ["tests/fixtures/**", "docs/generated/**"]
rules = ["jwt"]

[[rules]]
id = "internal-service-token"
description = "Internal production service token"
regex = "svc_live_[A-Za-z0-9]{40}"
```

For a one-line false positive, append `pushveil:allow` to that line. Prefer narrow custom rules and path exceptions; never allowlist a real credential. Full precedence and syntax are documented in [docs/configuration.md](docs/configuration.md).

## Security model and limitations

This tool is an early, local safety barrier—not an unbypassable policy engine.

- Git intentionally allows `git push --no-verify`, which skips `pre-push` hooks. A user can also replace local configuration or the binary.
- Client-side scanning cannot protect pushes made from machines where the tool is not installed.
- No finite rule set detects every possible secret. Random values without recognizable format or sensitive context can be indistinguishable from normal data.
- Raw binary scanning detects plaintext credential bytes, but it does not decrypt files or recursively unpack every archive/container format. Encrypted, obfuscated, compressed-only, generated, or split credentials may evade detection.
- If a real secret was committed, deleting it in a later commit does not remove it from history. Revoke/rotate it immediately, then rewrite history if appropriate.

For organization-wide enforcement, pair Pushveil with server-side pre-receive controls or the hosting provider's push-protection and secret-scanning features. The local tool provides fast developer feedback; the server remains the authoritative boundary.

## Performance and privacy

The scanner starts only during a push or explicit scan. Git objects are streamed through one long-lived `git cat-file --batch` process and deduplicated by object ID. Memory remains bounded by the scan buffer plus findings, rather than repository size. There are no network calls and secret values are not written to terminal output or an application log.

## Enterprise deployment

Distribute a signed release binary with Intune, Jamf, Group Policy, an MDM, or a configuration-management system, then run `pushveil install` in each developer's user context. A centrally managed global `config.toml` can add organization-specific rules and prohibit unsafe exceptions through filesystem policy. Run `pushveil doctor` as a compliance check.

Do not treat workstation deployment alone as mandatory enforcement; use a server-side layer as described above.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

Integration tests create isolated local repositories and bare remotes. They never modify the developer's real global Git configuration.

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), [docs/architecture.md](docs/architecture.md), and the [publishing checklist](docs/publishing.md).

## License

MIT. See [LICENSE](LICENSE).
