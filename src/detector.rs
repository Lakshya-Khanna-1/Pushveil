use std::collections::HashSet;
use std::io::Read;

use anyhow::{Context, Result};
use regex::bytes::Regex;
use serde::Serialize;

use crate::config::Config;

const CHUNK_SIZE: usize = 64 * 1024;
const OVERLAP: usize = 8 * 1024;
const PREFIX_LIMIT: usize = 2 * 1024;
const INLINE_ALLOW: &[u8] = b"pushveil:allow";

#[derive(Debug, Clone)]
struct Rule {
    id: String,
    description: String,
    regex: Regex,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub path: String,
    pub commit: String,
    pub object_id: String,
    pub lfs: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule: String,
    pub description: String,
    pub path: String,
    pub commit: String,
    pub object_id: String,
    pub line: Option<u64>,
    pub byte_offset: u64,
    pub binary: bool,
    pub lfs: bool,
}

#[derive(Debug)]
pub struct StreamResult {
    pub findings: Vec<Finding>,
    pub bytes_scanned: u64,
    pub prefix: Vec<u8>,
}

pub struct Detector {
    rules: Vec<Rule>,
    allowed_rules: HashSet<String>,
}

impl Detector {
    pub fn new(config: &Config) -> Result<Self> {
        let mut rules = Vec::new();
        for (id, description, pattern) in builtin_rules() {
            rules.push(Rule {
                id: (*id).to_owned(),
                description: (*description).to_owned(),
                regex: Regex::new(pattern)
                    .with_context(|| format!("invalid built-in rule `{id}`"))?,
            });
        }
        for custom in &config.rules {
            rules.push(Rule {
                id: custom.id.clone(),
                description: custom.description.clone(),
                regex: Regex::new(&custom.regex)
                    .with_context(|| format!("invalid custom rule `{}`", custom.id))?,
            });
        }
        Ok(Self {
            rules,
            allowed_rules: config.allowlist.rules.iter().cloned().collect(),
        })
    }

    pub fn scan<R: Read>(&self, mut reader: R, source: &Source) -> Result<StreamResult> {
        let mut buffer = vec![0_u8; CHUNK_SIZE];
        let mut tail = Vec::new();
        let mut prefix = Vec::new();
        let mut findings = Vec::new();
        let mut total_read = 0_u64;
        let mut tail_line = 1_u64;
        let mut emitted_through = 0_u64;
        let mut binary = false;

        loop {
            let count = reader
                .read(&mut buffer)
                .context("could not read Git object")?;
            if count == 0 {
                break;
            }
            if prefix.len() < PREFIX_LIMIT {
                let remaining = PREFIX_LIMIT - prefix.len();
                prefix.extend_from_slice(&buffer[..count.min(remaining)]);
                binary |= buffer[..count.min(remaining)].contains(&0);
            }

            let base_offset = total_read.saturating_sub(tail.len() as u64);
            let mut window = Vec::with_capacity(tail.len() + count);
            window.extend_from_slice(&tail);
            window.extend_from_slice(&buffer[..count]);
            total_read += count as u64;

            let safe_end = window.len().saturating_sub(OVERLAP);
            self.scan_window(
                &window,
                safe_end,
                base_offset,
                tail_line,
                emitted_through,
                binary,
                source,
                &mut findings,
            );
            emitted_through = base_offset + safe_end as u64;

            let new_tail_start = window.len().saturating_sub(OVERLAP);
            tail_line += count_newlines(&window[..new_tail_start]);
            tail.clear();
            tail.extend_from_slice(&window[new_tail_start..]);
        }

        let base_offset = total_read.saturating_sub(tail.len() as u64);
        self.scan_window(
            &tail,
            tail.len(),
            base_offset,
            tail_line,
            emitted_through,
            binary,
            source,
            &mut findings,
        );

        Ok(StreamResult {
            findings,
            bytes_scanned: total_read,
            prefix,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_window(
        &self,
        window: &[u8],
        safe_end: usize,
        base_offset: u64,
        base_line: u64,
        emitted_through: u64,
        binary: bool,
        source: &Source,
        findings: &mut Vec<Finding>,
    ) {
        for rule in &self.rules {
            if self.allowed_rules.contains(&rule.id) {
                continue;
            }
            for matched in rule.regex.find_iter(window) {
                let absolute_end = base_offset + matched.end() as u64;
                if matched.end() > safe_end || absolute_end <= emitted_through {
                    continue;
                }
                let bytes = matched.as_bytes();
                if looks_like_placeholder(bytes) || line_has_inline_allow(window, matched.start()) {
                    continue;
                }
                let absolute_start = base_offset + matched.start() as u64;
                let line =
                    (!binary).then(|| base_line + count_newlines(&window[..matched.start()]));
                if rule.id == "generic-secret"
                    && findings.iter().any(|finding| {
                        finding.object_id == source.object_id && finding.line == line
                    })
                {
                    continue;
                }
                findings.push(Finding {
                    rule: rule.id.clone(),
                    description: rule.description.clone(),
                    path: source.path.clone(),
                    commit: source.commit.clone(),
                    object_id: source.object_id.clone(),
                    line,
                    byte_offset: absolute_start,
                    binary,
                    lfs: source.lfs,
                });
            }
        }
    }
}

fn count_newlines(bytes: &[u8]) -> u64 {
    bytecount::count(bytes, b'\n') as u64
}

fn line_has_inline_allow(window: &[u8], position: usize) -> bool {
    let start = window[..position]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |index| index + 1);
    let end = window[position..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(window.len(), |index| position + index);
    window[start..end]
        .windows(INLINE_ALLOW.len())
        .any(|candidate| candidate.eq_ignore_ascii_case(INLINE_ALLOW))
}

fn looks_like_placeholder(bytes: &[u8]) -> bool {
    let lowercase = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    [
        "placeholder",
        "replace_me",
        "replace-me",
        "your_api_key",
        "your-api-key",
        "example_key",
        "example-key",
        "not-a-real",
        "redacted",
        "changeme",
    ]
    .iter()
    .any(|marker| lowercase.contains(marker))
}

const fn builtin_rules() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        (
            "aws-access-key",
            "AWS access key ID",
            r"(?:AKIA|ASIA)[A-Z0-9]{16}",
        ),
        (
            "github-token",
            "GitHub access token",
            r"(?:gh[pousr]_[A-Za-z0-9]{36,255}|github_pat_[A-Za-z0-9_]{22,255})",
        ),
        (
            "gitlab-token",
            "GitLab access token",
            r"glpat-[A-Za-z0-9_-]{20,}",
        ),
        (
            "openai-key",
            "OpenAI API key",
            r"(?:sk-(?:proj-|svcacct-)[A-Za-z0-9_-]{20,}|sk-[A-Za-z0-9]{32,})",
        ),
        (
            "anthropic-key",
            "Anthropic API key",
            r"sk-ant-[A-Za-z0-9_-]{20,}",
        ),
        ("google-api-key", "Google API key", r"AIza[0-9A-Za-z_-]{35}"),
        (
            "stripe-secret-key",
            "Stripe secret key",
            r"(?:sk|rk)_(?:live|test)_[0-9A-Za-z]{16,}",
        ),
        (
            "slack-token",
            "Slack token",
            r"xox[baprs]-[0-9A-Za-z-]{10,}",
        ),
        ("npm-token", "npm access token", r"npm_[A-Za-z0-9]{36}"),
        (
            "pypi-token",
            "PyPI upload token",
            r"pypi-AgEIcHlwaS5vcmc[A-Za-z0-9_-]{20,}",
        ),
        (
            "sendgrid-key",
            "SendGrid API key",
            r"SG\.[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{20,}",
        ),
        ("twilio-key", "Twilio API key", r"SK[0-9a-fA-F]{32}"),
        (
            "private-key",
            "Private key",
            r"-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY(?: BLOCK)?-----",
        ),
        (
            "jwt",
            "JSON Web Token",
            r"eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}",
        ),
        (
            "credential-url",
            "Credential embedded in URL",
            r"(?i)(?:https?|postgres(?:ql)?|mysql|mongodb(?:\+srv)?|redis)://[^\s/:]{1,64}:[^\s/@]{8,128}@",
        ),
        (
            "generic-secret",
            "Secret assigned to a sensitive variable",
            r#"(?i)(?:api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret|secret[_-]?key|aws[_-]?secret[_-]?access[_-]?key|password|passwd)[\t ]{0,12}(?:=|:)[\t ]{0,12}[\"']?[A-Za-z0-9_./+=:@-]{12,512}"#,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Source {
        Source {
            path: "src/config.rs".into(),
            commit: "abc123".into(),
            object_id: "def456".into(),
            lfs: false,
        }
    }

    #[test]
    fn detects_a_key_split_across_chunks() {
        let detector = Detector::new(&Config::default()).unwrap();
        let mut content = vec![b'a'; CHUNK_SIZE - 8];
        content.extend_from_slice(b"\nOPENAI_API_KEY=sk-proj-");
        content.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz0123456789\n");
        let result = detector.scan(content.as_slice(), &source()).unwrap();
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.rule == "openai-key")
        );
    }

    #[test]
    fn scans_binary_content() {
        let detector = Detector::new(&Config::default()).unwrap();
        let mut content = b"\0\x01AKIA".to_vec();
        content.extend_from_slice(b"ABCDEFGHIJKLMNOP\x02");
        let result = detector.scan(content.as_slice(), &source()).unwrap();
        assert!(result.findings[0].binary);
        assert_eq!(result.findings[0].line, None);
    }

    #[test]
    fn honors_inline_allow_marker() {
        let detector = Detector::new(&Config::default()).unwrap();
        let mut content = b"token = sk-proj-".to_vec();
        content.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz # pushveil:allow\n");
        let result = detector.scan(content.as_slice(), &source()).unwrap();
        assert!(result.findings.is_empty());
    }
}
