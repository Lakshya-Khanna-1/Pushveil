# Installation

Pushveil is a single native executable. Git is its only runtime dependency. Rust is required only when building from source.

## Requirements

- Git 2.x available on `PATH`
- A supported desktop or server operating system
- Permission to update the current user's global Git configuration

Administrator or root privileges are not required for a user installation.

## Install a release build

Download the archive matching your operating system and CPU from the repository's [Releases page](https://github.com/Lakshya-Khanna-1/Pushveil/releases), verify its SHA-256 checksum, and extract it.

On Windows PowerShell:

```powershell
.\pushveil.exe install
.\pushveil.exe doctor
```

On macOS or Linux:

```bash
chmod +x ./pushveil
./pushveil install
./pushveil doctor
```

`doctor` should report that Git is available, the installed binary exists, the `pre-push` wrapper exists, and global hook routing is active.

## Build from source

Building requires Rust 1.85 or newer:

```bash
git clone https://github.com/Lakshya-Khanna-1/Pushveil.git
cd Pushveil
cargo build --release --locked
./target/release/pushveil install
```

Windows uses `target\release\pushveil.exe`.

## What installation changes

Pushveil copies the running executable to a versioned file inside the current user's application-config directory. It creates a hook-routing directory and records the previous global `core.hooksPath` value in `install-state.toml`. It then points Git's global `core.hooksPath` at the Pushveil wrappers.

Wrappers are installed for standard Git hook names so existing repository hooks keep working. Only `pre-push` performs a security scan; other wrappers dispatch to the hook configuration that existed before Pushveil.

## Installation scope

Installation applies to one operating-system user and one Git environment. Repeat it separately for:

- another user account;
- WSL distributions;
- Docker or development containers;
- virtual machines;
- SSH hosts;
- remote or cloud coding-agent workers.

Installing on the Windows host does not configure Git inside WSL.

## Upgrade

Download the newer binary and run `pushveil install` again. The installer writes a versioned executable, refreshes the wrappers, and preserves the original pre-Pushveil hook path rather than chaining an older Pushveil installation.

## Uninstall

```bash
pushveil uninstall
```

Uninstall restores the previous global hook configuration only when Git still points to Pushveil. This avoids overwriting a hook path the user changed after installation. Versioned binaries remain cached because Windows cannot reliably delete the executable currently performing the uninstall; they are inert once Git routing is restored.

