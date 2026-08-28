mod cli;
mod config;
mod detector;
mod git;
mod hook;
mod install;
mod report;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("pushveil: {error:#}");
            ExitCode::FAILURE
        }
    }
}
