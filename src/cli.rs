use std::ffi::OsString;
use std::io::{self, Read};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::{config::Config, git::Git, hook, install, report};

#[derive(Debug, Parser)]
#[command(
    name = "pushveil",
    version,
    about = "Stop secrets before they leave your computer",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Install the scanner for every Git repository owned by this user.
    Install,
    /// Restore the previous global Git hook configuration.
    Uninstall,
    /// Verify the installation and local prerequisites.
    Doctor,
    /// Scan Git history without pushing.
    Scan {
        /// Revision or revision range accepted by `git rev-list`.
        #[arg(default_value = "HEAD")]
        revision: String,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Internal Git-hook entry point.
    #[command(hide = true)]
    Hook {
        hook_name: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<OsString>,
    },
}

pub fn run() -> Result<u8> {
    let cli = Cli::parse();
    match cli.command {
        Command::Install => {
            install::install()?;
            Ok(0)
        }
        Command::Uninstall => {
            install::uninstall()?;
            Ok(0)
        }
        Command::Doctor => Ok(install::doctor()?),
        Command::Scan { revision, json } => scan(&revision, json),
        Command::Hook { hook_name, args } => hook::run(&hook_name, &args),
    }
}

fn scan(revision: &str, json: bool) -> Result<u8> {
    let git = Git::discover()?;
    let config = Config::load(&git)?;
    let commits = git
        .rev_list(&[revision.to_owned()])
        .with_context(|| format!("could not resolve revision `{revision}`"))?;
    let result = git.scan_commits(&commits, &config)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        report::print_scan_result(&result, config.scan.max_findings_shown);
    }

    Ok(u8::from(
        !result.findings.is_empty() || (config.scan.fail_closed && !result.errors.is_empty()),
    ))
}

pub fn read_stdin_bytes() -> Result<Vec<u8>> {
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .context("could not read hook input")?;
    Ok(input)
}
