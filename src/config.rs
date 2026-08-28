use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;

use crate::git::Git;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub scan: ScanConfig,
    pub allowlist: AllowlistConfig,
    pub rules: Vec<CustomRule>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScanConfig {
    pub lfs: bool,
    pub fail_closed: bool,
    pub max_findings_shown: usize,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AllowlistConfig {
    pub paths: Vec<String>,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomRule {
    pub id: String,
    pub description: String,
    pub regex: String,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            lfs: true,
            fail_closed: true,
            max_findings_shown: 100,
        }
    }
}

impl Config {
    pub fn load(git: &Git) -> Result<Self> {
        let global = config_dir().join("config.toml");
        let repository = git.work_tree().map(|root| root.join(".pushveil.toml"));

        let mut config = if global.exists() {
            Self::from_path(&global)?
        } else {
            Self::default()
        };

        if let Some(path) = repository.filter(|path| path.exists()) {
            let repository_config = Self::from_path(&path)?;
            config.scan = repository_config.scan;
            config.allowlist = repository_config.allowlist;
            config.rules.extend(repository_config.rules);
        }

        if config.scan.max_findings_shown == 0 {
            config.scan.max_findings_shown = 1;
        }
        Ok(config)
    }

    pub fn path_allowlist(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &self.allowlist.paths {
            builder.add(
                Glob::new(pattern)
                    .with_context(|| format!("invalid allowlisted path glob `{pattern}`"))?,
            );
        }
        builder.build().context("could not build path allowlist")
    }

    fn from_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("could not read configuration at {}", path.display()))?;
        toml::from_str(&text)
            .with_context(|| format!("invalid configuration at {}", path.display()))
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("PUSHVEIL_HOME") {
        return PathBuf::from(path);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pushveil")
}
