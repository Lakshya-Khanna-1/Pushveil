# AI coding agents

Pushveil protects the Git execution environment, not a specific editor or agent. No agent plugin is required.

## Supported workflow

Codex, Claude Code, Antigravity, Cursor, Windsurf, Cline, Aider, Gemini CLI, and similar agents are protected when they run the normal system `git push` command under the user account where Pushveil is installed.

```text
coding agent
  → system git push
  → global pre-push hook
  → Pushveil scan
  → clean push or nonzero blocked result
```

The agent receives masked findings through the same standard error stream a human sees. It can use the file, line, commit, and rule ID to remove the problem and create corrected commits.

## Non-interactive safety

An agent process commonly captures terminal output without an attached interactive terminal. Pushveil detects that condition and chooses cancellation. It does not read an override from the hook's standard input, because that stream contains Git ref updates and may be controlled by automation.

Only a human in an interactive terminal can type `PUSH ANYWAY`.

## Agent instructions

Add guidance like this to an agent policy file:

```text
Use the system Git executable for all pushes. Never pass --no-verify.
If Pushveil blocks a push, do not override it. Remove the credential from
every affected unpushed commit, rotate any real key, and rerun the push.
Run pushveil doctor if hook protection appears unavailable.
```

## Verify the environment

Ask the agent to run:

```bash
pushveil doctor
git config --global --get core.hooksPath
git --version
```

The environment needs its own installation when it has a separate user profile, Git configuration, or filesystem.

## Containers, WSL, and remote agents

Host installation does not cross isolation boundaries. Install Pushveil inside each WSL distribution, development container, virtual machine, SSH host, or cloud sandbox that can push. Persist both the executable/config directory and global Git configuration when containers are recreated.

## Unsupported bypass paths

Local Git hooks do not run when an agent:

- uses `git push --no-verify`;
- updates refs through a GitHub/GitLab API;
- uses an embedded Git implementation that ignores native hooks;
- sends the push from another unprotected machine;
- delegates pushing to a hosted service.

Use hosting-provider push protection or a server-side pre-receive hook for mandatory coverage.

## Commits created by agents

It does not matter who authored the commit. If an agent writes and commits code but a person later runs the protected `git push`, Pushveil scans the agent-created history at that moment.

