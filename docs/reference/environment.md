# Files, configuration, and environment reference

## Repository file

| File | Purpose |
|---|---|
| `.pushveil.toml` | Repository scan settings, allowlists, and custom rules |

The file is read from the worktree root associated with the active Git repository.

## User application directory

Default locations:

| Platform | Directory |
|---|---|
| Windows | `%APPDATA%\pushveil` |
| macOS | `~/Library/Application Support/pushveil` |
| Linux | `${XDG_CONFIG_HOME:-~/.config}/pushveil` |

Contents include:

- `config.toml`: optional user configuration;
- `install-state.toml`: prior hook path, installed binary path, hooks path, and version;
- `bin/pushveil-<version>[.exe]`: versioned installed executable;
- `hooks/`: global dispatch wrappers.

## Environment variables

### `PUSHVEIL_HOME`

Overrides the entire Pushveil application directory. It is useful for portable installations, managed layouts, and isolated tests.

```bash
PUSHVEIL_HOME=/opt/company/pushveil pushveil install
```

The variable must be present during both installation and later hook execution. A wrapper points directly to the installed binary, but configuration and state lookup still use the effective Pushveil home.

### `NO_COLOR`

When present, disables ANSI color in human-readable output. Pushveil also avoids color when standard error is not a terminal.

```bash
NO_COLOR=1 pushveil scan HEAD
```

## Git configuration

Installation writes one user-level key:

```bash
git config --global core.hooksPath <pushveil-hooks-directory>
```

The prior value is stored before replacement and restored by uninstall. Repository-local `core.hooksPath` can override a global value under Git's normal configuration precedence and may therefore bypass global protection; enterprise controls must account for this.

## Repository hook directory

When no prior global hooks path existed, Pushveil dispatches existing hooks from the active Git directory's `hooks` folder. When a prior global path existed, it dispatches that path instead, matching the behavior in place before installation.

## Standard streams

- Human findings and status summaries use standard error so they remain visible during `git push`.
- JSON manual-scan output uses standard output.
- The pre-push hook's standard input contains Git ref-update records and is preserved for chained hooks.

## Path and portability notes

Generated wrappers use an absolute executable path and shell-safe quoting. Git for Windows executes shebang hooks through `sh` when necessary. Unix wrappers receive executable permissions during installation.
