use std::io::{self, IsTerminal};

use crate::git::ScanResult;

pub fn print_scan_result(result: &ScanResult, max_findings: usize) {
    let color = io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    if result.findings.is_empty() && result.errors.is_empty() {
        eprintln!(
            "{} scanned {} commit(s), {} blob(s), {} — no secrets found",
            green("✓", color),
            result.stats.commits,
            result.stats.blobs,
            human_bytes(result.stats.bytes)
        );
        if result.stats.submodules > 0 {
            eprintln!(
                "  {} submodule reference(s) found; their contents are checked by pushes from each submodule repository",
                result.stats.submodules
            );
        }
        return;
    }

    if !result.findings.is_empty() {
        eprintln!(
            "\n{} Push blocked: {} potential secret(s) found\n",
            red("✗", color),
            result.findings.len()
        );
        for finding in result.findings.iter().take(max_findings) {
            let location = finding.line.map_or_else(
                || format!("{} (byte {})", finding.path, finding.byte_offset),
                |line| format!("{}:{line}", finding.path),
            );
            let commit = &finding.commit[..finding.commit.len().min(12)];
            let source = if finding.lfs {
                "Git LFS object"
            } else {
                "Git blob"
            };
            eprintln!("  {}  {}", yellow(&finding.description, color), location);
            eprintln!("      commit {commit} · {source} · rule {}", finding.rule);
        }
        if result.findings.len() > max_findings {
            eprintln!(
                "  … {} additional finding(s) hidden; increase scan.max_findings_shown to display them",
                result.findings.len() - max_findings
            );
        }
        eprintln!("\nSecret values are intentionally masked from terminal output.");
    }

    if !result.errors.is_empty() {
        eprintln!(
            "\n{} Scanner could not verify all content:",
            red("!", color)
        );
        for error in &result.errors {
            eprintln!("  - {error}");
        }
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes;
    let mut unit = 0;
    let mut remainder = 0;
    while value >= 1024 && unit < UNITS.len() - 1 {
        remainder = value % 1024;
        value /= 1024;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        let decimal = remainder.saturating_mul(10) / 1024;
        format!("{value}.{decimal} {}", UNITS[unit])
    }
}

fn red(value: &str, enabled: bool) -> String {
    paint(value, "31;1", enabled)
}

fn green(value: &str, enabled: bool) -> String {
    paint(value, "32;1", enabled)
}

fn yellow(value: &str, enabled: bool) -> String {
    paint(value, "33;1", enabled)
}

fn paint(value: &str, code: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{code}m{value}\x1b[0m")
    } else {
        value.to_owned()
    }
}
