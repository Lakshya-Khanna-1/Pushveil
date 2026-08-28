use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::config_dir;

const HOOK_NAMES: &[&str] = &[
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "proc-receive",
    "post-receive",
    "post-update",
    "reference-transaction",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallState {
    pub previous_hooks_path: Option<String>,
    pub hooks_path: PathBuf,
    pub binary_path: PathBuf,
    pub version: String,
}

pub fn install() -> Result<()> {
    verify_git()?;
    let root = config_dir();
    let hooks_path = root.join("hooks");
    let binary_name = if cfg!(windows) {
        format!("pushveil-{}.exe", env!("CARGO_PKG_VERSION"))
    } else {
        format!("pushveil-{}", env!("CARGO_PKG_VERSION"))
    };
    let binary_path = root.join("bin").join(binary_name);
    fs::create_dir_all(binary_path.parent().expect("binary has parent"))?;
    fs::create_dir_all(&hooks_path)?;

    let current_exe = std::env::current_exe().context("could not locate current executable")?;
    if !same_file_path(&current_exe, &binary_path) {
        fs::copy(&current_exe, &binary_path).with_context(|| {
            format!(
                "could not install executable from {} to {}",
                current_exe.display(),
                binary_path.display()
            )
        })?;
    }

    let previous_state = load_state().ok();
    let current_hooks_path = git_config_get("core.hooksPath")?;
    let hooks_string = hooks_path.to_string_lossy().into_owned();
    let previous_hooks_path = if current_hooks_path.as_deref() == Some(hooks_string.as_str()) {
        previous_state.and_then(|state| state.previous_hooks_path)
    } else {
        current_hooks_path
    };

    for hook_name in HOOK_NAMES {
        write_wrapper(&hooks_path.join(hook_name), &binary_path, hook_name)?;
    }

    let state = InstallState {
        previous_hooks_path,
        hooks_path: hooks_path.clone(),
        binary_path,
        version: env!("CARGO_PKG_VERSION").into(),
    };
    write_state(&state)?;
    git_config_set("core.hooksPath", hooks_path.as_os_str())?;

    println!(
        "Pushveil {} installed successfully.",
        env!("CARGO_PKG_VERSION")
    );
    println!("Global Git hooks: {}", hooks_path.display());
    println!("Every push from this user account will now be scanned.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let state = load_state().context("Pushveil is not installed")?;
    let current = git_config_get("core.hooksPath")?;
    let installed = state.hooks_path.to_string_lossy();
    if current.as_deref() != Some(installed.as_ref()) {
        bail!(
            "Git's current core.hooksPath is `{}`; refusing to replace a configuration that no longer belongs to Pushveil",
            current.as_deref().unwrap_or("<unset>")
        );
    }

    if let Some(previous) = &state.previous_hooks_path {
        git_config_set("core.hooksPath", OsStr::new(previous))?;
        println!("Restored the previous global hooks path: {previous}");
    } else {
        git_config_unset("core.hooksPath")?;
        println!("Restored Git's repository-local hook behavior.");
    }
    println!(
        "Pushveil is disabled. Cached program files remain at {} and are harmless.",
        config_dir().display()
    );
    Ok(())
}

pub fn doctor() -> Result<u8> {
    let mut healthy = true;
    match verify_git() {
        Ok(()) => println!("[ok] Git is available"),
        Err(error) => {
            println!("[error] Git is unavailable: {error:#}");
            healthy = false;
        }
    }

    if let Ok(state) = load_state() {
        if state.binary_path.is_file() {
            println!("[ok] Installed binary: {}", state.binary_path.display());
        } else {
            println!(
                "[error] Installed binary is missing: {}",
                state.binary_path.display()
            );
            healthy = false;
        }
        if state.hooks_path.join("pre-push").is_file() {
            println!("[ok] Pre-push hook is installed");
        } else {
            println!("[error] Pre-push hook is missing");
            healthy = false;
        }
        let configured = git_config_get("core.hooksPath")?;
        if configured.as_deref() == Some(state.hooks_path.to_string_lossy().as_ref()) {
            println!("[ok] Git global hook routing is active");
        } else {
            println!("[error] Git global hook routing points elsewhere");
            healthy = false;
        }
    } else {
        println!("[error] Pushveil is not installed; run `pushveil install`");
        healthy = false;
    }
    Ok(u8::from(!healthy))
}

pub fn load_state() -> Result<InstallState> {
    let path = state_path();
    let text = fs::read_to_string(&path)
        .with_context(|| format!("could not read install state at {}", path.display()))?;
    toml::from_str(&text).context("invalid install state")
}

fn write_state(state: &InstallState) -> Result<()> {
    let path = state_path();
    let text = toml::to_string_pretty(state)?;
    fs::write(&path, text)
        .with_context(|| format!("could not write install state at {}", path.display()))
}

fn state_path() -> PathBuf {
    config_dir().join("install-state.toml")
}

fn write_wrapper(path: &Path, binary: &Path, hook_name: &str) -> Result<()> {
    let binary = binary.to_string_lossy().replace('\\', "/");
    let binary = binary.replace('\'', "'\\''");
    let content = format!("#!/bin/sh\nexec '{binary}' hook '{hook_name}' \"$@\"\n");
    fs::write(path, content)
        .with_context(|| format!("could not write hook wrapper at {}", path.display()))?;
    set_executable(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn verify_git() -> Result<()> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .context("could not start Git; install Git and ensure it is on PATH")?;
    if !output.status.success() {
        bail!("`git --version` failed");
    }
    Ok(())
}

fn git_config_get(key: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .context("could not read global Git configuration")?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        ))
    } else if output.status.code() == Some(1) {
        Ok(None)
    } else {
        bail!(
            "could not read global Git configuration: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}

fn git_config_set(key: &str, value: &OsStr) -> Result<()> {
    let output = Command::new("git")
        .args([
            OsStr::new("config"),
            OsStr::new("--global"),
            OsStr::new(key),
            value,
        ])
        .output()
        .context("could not update global Git configuration")?;
    if !output.status.success() {
        bail!(
            "could not update global Git configuration: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn git_config_unset(key: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["config", "--global", "--unset-all", key])
        .output()
        .context("could not update global Git configuration")?;
    if output.status.success() || output.status.code() == Some(5) || output.status.code() == Some(1)
    {
        Ok(())
    } else {
        bail!(
            "could not remove global Git configuration: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
    }
}

fn same_file_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
