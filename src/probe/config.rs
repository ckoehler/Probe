use argh::FromArgs;
use regex::Error as RegexError;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid regex for probe '{probe_name}': {source}")]
    InvalidRegex {
        probe_name: String,
        source: RegexError,
    },
    #[error("Duplicate probe name found: {name}")]
    DuplicateProbeName { name: String },
}

/// Probe Config
#[derive(Debug, FromArgs)]
pub struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "1000")]
    pub tick_rate: u64,
    /// config file.
    #[argh(option, default = "String::from(\"probe.toml\")")]
    pub config: String,
}

#[derive(Debug, Deserialize)]
pub struct Probes {
    pub probes: Vec<ProbeConfig>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct ProbeConfig {
    pub name: String,
    pub filter: Option<String>,
    pub address: String,
}

impl Probes {
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut names = HashSet::new();
        for probe in &self.probes {
            probe.validate()?;

            // Check for duplicate names
            if !names.insert(&probe.name) {
                return Err(ConfigError::DuplicateProbeName {
                    name: probe.name.clone(),
                });
            }
        }
        Ok(())
    }
}
impl ProbeConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        let pattern = self.filter.as_deref().unwrap_or(".*");
        Regex::new(pattern).map_err(|e| ConfigError::InvalidRegex {
            probe_name: self.name.clone(),
            source: e,
        })?;
        Ok(())
    }
}
