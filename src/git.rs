use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;

use crate::config::Config;
use crate::detector::{Detector, Finding, Source};

#[derive(Debug, Clone)]
pub struct Git {
    work_tree: Option<PathBuf>,
    invocation_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PushUpdate {
    pub local_object: String,
    pub remote_object: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ScanStats {
    pub commits: usize,
    pub blobs: usize,
    pub lfs_objects: usize,
    pub submodules: usize,
    pub bytes: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub errors: Vec<String>,
    pub stats: ScanStats,
}

#[derive(Debug, Clone)]
struct Blob {
    object_id: String,
    path: String,
    commit: String,
}

impl Git {
    pub fn discover() -> Result<Self> {
        let invocation_dir = std::env::current_dir().context("could not read current directory")?;
        let probe = git_output_at(&invocation_dir, ["rev-parse", "--show-toplevel"]);
        let work_tree = if let Ok(output) = probe {
            Some(PathBuf::from(output.trim()))
        } else {
            git_output_at(&invocation_dir, ["rev-parse", "--git-dir"])
                .context("not inside a Git repository")?;
            None
        };
        Ok(Self {
            work_tree,
            invocation_dir,
        })
    }

    pub fn work_tree(&self) -> Option<&Path> {
        self.work_tree.as_deref()
    }

    pub fn git_dir(&self) -> Result<PathBuf> {
        let output = self.output(["rev-parse", "--absolute-git-dir"])?;
        Ok(PathBuf::from(output.trim()))
    }

    pub fn parse_push_updates(input: &[u8]) -> Result<Vec<PushUpdate>> {
        let text = std::str::from_utf8(input).context("Git supplied non-UTF-8 push metadata")?;
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                if fields.len() != 4 {
                    bail!("invalid pre-push update line: `{line}`");
                }
                Ok(PushUpdate {
                    local_object: fields[1].to_owned(),
                    remote_object: fields[3].to_owned(),
                })
            })
            .collect()
    }

    pub fn commits_for_push(
        &self,
        updates: &[PushUpdate],
        remote_name: Option<&OsStr>,
    ) -> Result<Vec<String>> {
        let remote_name = remote_name.and_then(OsStr::to_str).unwrap_or_default();
        let mut seen = HashSet::new();
        let mut commits = Vec::new();

        for update in updates {
            if is_zero_id(&update.local_object) {
                continue;
            }
            let mut revisions = vec![update.local_object.clone()];
            if !is_zero_id(&update.remote_object) {
                revisions.push("--not".into());
                revisions.push(update.remote_object.clone());
            } else if !remote_name.is_empty() && self.remote_has_tracking_refs(remote_name)? {
                revisions.push("--not".into());
                revisions.push(format!("--remotes={remote_name}"));
            }

            for commit in self.rev_list(&revisions)? {
                if seen.insert(commit.clone()) {
                    commits.push(commit);
                }
            }
        }
        Ok(commits)
    }

    pub fn rev_list(&self, revisions: &[String]) -> Result<Vec<String>> {
        let mut args = vec![
            "rev-list".to_owned(),
            "--reverse".into(),
            "--topo-order".into(),
        ];
        args.extend(revisions.iter().cloned());
        let output = self.output(args.iter().map(String::as_str))?;
        Ok(output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect())
    }

    pub fn scan_commits(&self, commits: &[String], config: &Config) -> Result<ScanResult> {
        let detector = Detector::new(config)?;
        let allowlisted_paths = config.path_allowlist()?;
        let mut result = ScanResult::default();
        result.stats.commits = commits.len();

        let mut seen = HashSet::new();
        let mut blobs = Vec::new();
        for commit in commits {
            let (changed, submodules) = self.changed_blobs(commit)?;
            result.stats.submodules += submodules;
            for blob in changed {
                if allowlisted_paths.is_match(&blob.path) || !seen.insert(blob.object_id.clone()) {
                    continue;
                }
                blobs.push(blob);
            }
        }
        result.stats.blobs = blobs.len();

        let mut batch = CatFileBatch::start(self.command())?;
        for blob in blobs {
            let source = Source {
                path: blob.path.clone(),
                commit: blob.commit.clone(),
                object_id: blob.object_id.clone(),
                lfs: false,
            };
            let stream = batch
                .scan_blob(&blob.object_id, &detector, &source)
                .with_context(|| format!("could not scan {} ({})", blob.path, blob.object_id))?;
            result.stats.bytes += stream.bytes_scanned;
            result.findings.extend(stream.findings);

            if config.scan.lfs {
                if let Some(oid) = parse_lfs_pointer(&stream.prefix) {
                    match self.lfs_content_path(&oid) {
                        Ok(path) => {
                            let file = File::open(&path).with_context(|| {
                                format!("could not open Git LFS object at {}", path.display())
                            });
                            match file {
                                Ok(file) => {
                                    let lfs_source = Source {
                                        path: blob.path.clone(),
                                        commit: blob.commit.clone(),
                                        object_id: oid.clone(),
                                        lfs: true,
                                    };
                                    let lfs_result = detector.scan(file, &lfs_source)?;
                                    result.stats.lfs_objects += 1;
                                    result.stats.bytes += lfs_result.bytes_scanned;
                                    result.findings.extend(lfs_result.findings);
                                }
                                Err(error) => result.errors.push(format!("{error:#}")),
                            }
                        }
                        Err(error) => result.errors.push(format!(
                            "could not resolve Git LFS object {oid} for {}: {error:#}",
                            blob.path
                        )),
                    }
                }
            }
        }
        Ok(result)
    }

    fn changed_blobs(&self, commit: &str) -> Result<(Vec<Blob>, usize)> {
        let output = self.output_bytes([
            "diff-tree",
            "--root",
            "-r",
            "-m",
            "--no-commit-id",
            "--raw",
            "-z",
            "--no-renames",
            "--no-abbrev",
            commit,
        ])?;
        let fields: Vec<&[u8]> = output.split(|byte| *byte == 0).collect();
        let mut index = 0;
        let mut blobs = Vec::new();
        let mut submodules = 0;

        while index + 1 < fields.len() {
            let header = String::from_utf8_lossy(fields[index]);
            let path = String::from_utf8_lossy(fields[index + 1]).into_owned();
            index += 2;
            if header.trim().is_empty() {
                continue;
            }
            let parts: Vec<_> = header.split_whitespace().collect();
            if parts.len() < 5 {
                bail!("could not parse Git raw diff entry `{}`", header.trim());
            }
            let new_mode = parts[1];
            let new_object = parts[3];
            let status = parts[4];
            if status.starts_with('D') || is_zero_id(new_object) {
                continue;
            }
            if new_mode == "160000" {
                submodules += 1;
                continue;
            }
            blobs.push(Blob {
                object_id: new_object.to_owned(),
                path,
                commit: commit.to_owned(),
            });
        }
        Ok((blobs, submodules))
    }

    fn remote_has_tracking_refs(&self, remote_name: &str) -> Result<bool> {
        let prefix = format!("refs/remotes/{remote_name}/");
        Ok(!self
            .output(["for-each-ref", "--format=%(refname)", &prefix])?
            .trim()
            .is_empty())
    }

    fn lfs_content_path(&self, oid: &str) -> Result<PathBuf> {
        let environment = self.output(["lfs", "env"])?;
        let media_dir = environment
            .lines()
            .find_map(|line| line.strip_prefix("LocalMediaDir="))
            .filter(|value| !value.is_empty())
            .context("Git LFS did not report LocalMediaDir")?;
        let media_dir = PathBuf::from(media_dir);
        let media_dir = if media_dir.is_absolute() {
            media_dir
        } else {
            self.invocation_dir.join(media_dir)
        };
        Ok(media_dir.join(&oid[..2]).join(&oid[2..4]).join(oid))
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        command.current_dir(self.work_tree.as_ref().unwrap_or(&self.invocation_dir));
        command
    }

    fn output<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = self.output_bytes(args)?;
        String::from_utf8(output).context("Git returned non-UTF-8 output")
    }

    fn output_bytes<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = self.command();
        command.args(args);
        let debug = format!("{command:?}");
        let output = command
            .output()
            .with_context(|| format!("could not run {debug}"))?;
        if !output.status.success() {
            bail!(
                "{debug} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(output.stdout)
    }
}

struct CatFileBatch {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl CatFileBatch {
    fn start(mut command: Command) -> Result<Self> {
        let mut child = command
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("could not start `git cat-file --batch`")?;
        let stdin = child.stdin.take().context("missing git cat-file stdin")?;
        let stdout = child.stdout.take().context("missing git cat-file stdout")?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn scan_blob(
        &mut self,
        object_id: &str,
        detector: &Detector,
        source: &Source,
    ) -> Result<crate::detector::StreamResult> {
        writeln!(self.stdin, "{object_id}")?;
        self.stdin.flush()?;

        let mut header = String::new();
        self.stdout.read_line(&mut header)?;
        let fields: Vec<_> = header.split_whitespace().collect();
        if fields.len() != 3 || fields[1] != "blob" {
            bail!("unexpected git cat-file response `{}`", header.trim());
        }
        let size: u64 = fields[2]
            .parse()
            .with_context(|| format!("invalid Git blob size in `{}`", header.trim()))?;
        let mut limited = self.stdout.by_ref().take(size);
        let result = detector.scan(&mut limited, source)?;
        std::io::copy(&mut limited, &mut std::io::sink())?;
        let mut delimiter = [0_u8; 1];
        self.stdout.read_exact(&mut delimiter)?;
        if delimiter[0] != b'\n' {
            return Err(anyhow!("invalid git cat-file record delimiter"));
        }
        Ok(result)
    }
}

impl Drop for CatFileBatch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn is_zero_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte == b'0')
}

fn parse_lfs_pointer(prefix: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(prefix).ok()?;
    if !text.starts_with("version https://git-lfs.github.com/spec/v1\n") {
        return None;
    }
    text.lines()
        .find_map(|line| line.strip_prefix("oid sha256:"))
        .filter(|oid| oid.len() == 64 && oid.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_owned)
}

fn git_output_at<I, S>(directory: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .current_dir(directory)
        .args(args)
        .output()
        .context("could not start Git")?;
    if !output.status.success() {
        bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }
    String::from_utf8(output.stdout).context("Git returned non-UTF-8 output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lfs_pointer() {
        let pointer = b"version https://git-lfs.github.com/spec/v1\noid sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\nsize 42\n";
        assert_eq!(parse_lfs_pointer(pointer).unwrap().len(), 64);
    }

    #[test]
    fn parses_pre_push_input() {
        let input = b"refs/heads/main abc refs/heads/main 000\n";
        let updates = Git::parse_push_updates(input).unwrap();
        assert_eq!(updates[0].local_object, "abc");
        assert_eq!(updates[0].remote_object, "000");
    }
}
