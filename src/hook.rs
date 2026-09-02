use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::cli::read_stdin_bytes;
use crate::config::Config;
use crate::git::Git;
use crate::{install, report};

pub fn run(hook_name: &str, args: &[OsString]) -> Result<u8> {
    assert_hook_name_is_safe(OsStr::new(hook_name))?;
    if hook_name == "pre-push" {
        run_pre_push(args)
    } else {
        run_chained_hook(hook_name, args, None)
    }
}

fn run_pre_push(args: &[OsString]) -> Result<u8> {
    let input = read_stdin_bytes()?;
    let git = Git::discover()?;
    let updates = Git::parse_push_updates(&input)?;
    let commits = git.commits_for_push(&updates, args.first().map(OsString::as_os_str))?;
    let config = Config::load(&git)?;
    let scan = git.scan_commits(&commits, &config)?;
    report::print_scan_result(&scan, config.scan.max_findings_shown);

    let blocked = !scan.findings.is_empty() || (config.scan.fail_closed && !scan.errors.is_empty());
    if blocked {
        eprintln!("\n[Enter or type OK] Cancel this push");
        eprintln!("[Type PUSH ANYWAY] Override once and continue");
        eprint!("> ");
        std::io::stderr().flush()?;
        let response = read_terminal_line()?;
        if !response.as_deref().is_some_and(is_override_confirmation) {
            eprintln!("Push cancelled. Remove the secret, commit the correction, and push again.");
            return Ok(1);
        }
        eprintln!("Override confirmed. Continuing this push with the reported risk.");
    }

    run_chained_hook("pre-push", args, Some(&input))
}

fn read_terminal_line() -> Result<Option<String>> {
    if !std::io::stderr().is_terminal() {
        eprintln!("No interactive terminal is available, so the safe default is to cancel.");
        return Ok(None);
    }

    #[cfg(windows)]
    let terminal = OpenOptions::new().read(true).open("CONIN$");
    #[cfg(unix)]
    let terminal = OpenOptions::new().read(true).open("/dev/tty");
    #[cfg(not(any(windows, unix)))]
    let terminal: std::io::Result<std::fs::File> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "terminal input is unsupported on this platform",
    ));

    if let Ok(file) = terminal {
        let mut line = String::new();
        BufReader::new(file).read_line(&mut line)?;
        Ok(Some(line))
    } else {
        eprintln!("No interactive terminal is available, so the safe default is to cancel.");
        Ok(None)
    }
}

fn is_override_confirmation(value: &str) -> bool {
    value.trim() == "PUSH ANYWAY"
}

fn run_chained_hook(hook_name: &str, args: &[OsString], input: Option<&[u8]>) -> Result<u8> {
    let Some(path) = chained_hook_path(hook_name) else {
        return Ok(0);
    };
    if !path.is_file() || !is_executable_hook(&path)? {
        return Ok(0);
    }

    let mut command = hook_command(&path);
    command
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if input.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::inherit());
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("could not run existing hook at {}", path.display()))?;
    if let Some(bytes) = input {
        child
            .stdin
            .take()
            .context("existing hook stdin was unavailable")?
            .write_all(bytes)?;
    }
    let status = child.wait()?;
    Ok(status
        .code()
        .map_or(1, |code| u8::try_from(code).unwrap_or(1)))
}

fn chained_hook_path(hook_name: &str) -> Option<PathBuf> {
    if let Ok(state) = install::load_state() {
        if let Some(previous) = state.previous_hooks_path {
            return Some(resolve_previous_path(&previous).join(hook_name));
        }
    }

    // Git invokes some hooks while a repository is still being initialized. In
    // that phase `git rev-parse` can fail even though GIT_DIR already identifies
    // where repository-local hooks belong. Prefer that environment value, and
    // otherwise fail open for passive hook forwarding when no repository exists.
    if let Some(git_dir) = std::env::var_os("GIT_DIR") {
        let git_dir = PathBuf::from(git_dir);
        let git_dir = if git_dir.is_absolute() {
            git_dir
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(git_dir)
        };
        return Some(git_dir.join("hooks").join(hook_name));
    }

    let Ok(git) = Git::discover() else {
        return None;
    };
    let Ok(git_dir) = git.git_dir() else {
        return None;
    };
    Some(git_dir.join("hooks").join(hook_name))
}

fn resolve_previous_path(value: &str) -> PathBuf {
    let expanded = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
        .map_or_else(
            || PathBuf::from(value),
            |rest| dirs::home_dir().map_or_else(|| PathBuf::from(value), |home| home.join(rest)),
        );
    if expanded.is_absolute() {
        expanded
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(expanded)
    }
}

fn hook_command(path: &Path) -> Command {
    #[cfg(windows)]
    {
        let mut prefix = [0_u8; 2];
        let is_script = std::fs::File::open(path)
            .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut prefix))
            .is_ok_and(|()| prefix == *b"#!");
        if is_script {
            let mut command = Command::new("sh");
            command.arg(path);
            return command;
        }
    }
    Command::new(path)
}

#[cfg(unix)]
fn is_executable_hook(path: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::metadata(path)?.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
fn is_executable_hook(_path: &Path) -> Result<bool> {
    Ok(true)
}

fn assert_hook_name_is_safe(value: &OsStr) -> Result<()> {
    let value = value.to_string_lossy();
    if value.contains('/') || value.contains('\\') || value.contains("..") {
        bail!("invalid hook name")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_override_confirmation;

    #[test]
    fn override_requires_the_exact_phrase() {
        assert!(is_override_confirmation("PUSH ANYWAY\n"));
        assert!(!is_override_confirmation("push anyway"));
        assert!(!is_override_confirmation("PUSH"));
        assert!(!is_override_confirmation("OK"));
    }
}
