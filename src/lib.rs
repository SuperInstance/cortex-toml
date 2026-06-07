//! # cortex-toml
//!
//! Configuration-as-code via `.cortex.toml` — parse, validate, serialize, diff,
//! and migrate exocortex configuration files.
//!
//! ## Core Types
//! - [`CortexConfig`] — Full configuration struct
//! - [`TomlParser`] — Parse `.cortex.toml` from string
//! - [`TomlSerializer`] — Serialize config back to TOML
//! - [`ConfigValidator`] — Validate config: check types, ranges, required fields
//! - [`ConfigDiff`] — Diff two configs, produce a migration plan

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full exocortex configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CortexConfig {
    /// Project name.
    pub name: String,
    /// Project version.
    #[serde(default)]
    pub version: String,
    /// Agent configurations.
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    /// Memory configuration.
    #[serde(default)]
    pub memory: MemoryConfig,
    /// Network configuration.
    #[serde(default)]
    pub network: NetworkConfig,
}

/// Configuration for a single agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// Agent type.
    #[serde(rename = "type")]
    pub agent_type: String,
    /// Agent model.
    #[serde(default)]
    pub model: String,
    /// Agent temperature (0.0–2.0).
    #[serde(default = "default_temperature")]
    pub temperature: f64,
}

fn default_temperature() -> f64 {
    0.7
}

/// Memory subsystem configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryConfig {
    /// Backend type ("file", "sqlite", etc.).
    #[serde(default = "default_memory_backend")]
    pub backend: String,
    /// Maximum memory entries.
    #[serde(default = "default_max_entries")]
    pub max_entries: u64,
    /// Decay factor.
    #[serde(default = "default_decay")]
    pub decay_factor: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            backend: default_memory_backend(),
            max_entries: default_max_entries(),
            decay_factor: default_decay(),
        }
    }
}

fn default_memory_backend() -> String {
    "file".to_string()
}

fn default_max_entries() -> u64 {
    10000
}

fn default_decay() -> f64 {
    0.95
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkConfig {
    /// Bind address.
    #[serde(default = "default_bind")]
    pub bind: String,
    /// Port number.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Enable TLS.
    #[serde(default)]
    pub tls: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind: default_bind(),
            port: default_port(),
            tls: false,
        }
    }
}

fn default_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

/// Parse `.cortex.toml` from a string.
pub struct TomlParser;

impl TomlParser {
    /// Parse a TOML string into a CortexConfig.
    pub fn parse(input: &str) -> Result<CortexConfig, String> {
        toml::from_str(input).map_err(|e| format!("Parse error: {}", e))
    }
}

/// Serialize a CortexConfig back to TOML.
pub struct TomlSerializer;

impl TomlSerializer {
    /// Serialize a config to a TOML string.
    pub fn serialize(config: &CortexConfig) -> Result<String, String> {
        toml::to_string_pretty(config).map_err(|e| format!("Serialize error: {}", e))
    }
}

/// A validation error.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Config validator.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a config, returning all errors found.
    pub fn validate(config: &CortexConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Name is required and non-empty
        if config.name.trim().is_empty() {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "Project name is required".to_string(),
            });
        }

        // Validate agents
        for (agent_name, agent) in &config.agents {
            if agent.agent_type.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("agents.{}.type", agent_name),
                    message: "Agent type is required".to_string(),
                });
            }
            if agent.temperature < 0.0 || agent.temperature > 2.0 {
                errors.push(ValidationError {
                    field: format!("agents.{}.temperature", agent_name),
                    message: format!("Temperature must be 0.0–2.0, got {}", agent.temperature),
                });
            }
        }

        // Validate memory
        if config.memory.max_entries == 0 {
            errors.push(ValidationError {
                field: "memory.max_entries".to_string(),
                message: "max_entries must be > 0".to_string(),
            });
        }
        if config.memory.decay_factor < 0.0 || config.memory.decay_factor > 1.0 {
            errors.push(ValidationError {
                field: "memory.decay_factor".to_string(),
                message: "decay_factor must be 0.0–1.0".to_string(),
            });
        }

        // Validate network
        if config.network.port == 0 {
            errors.push(ValidationError {
                field: "network.port".to_string(),
                message: "Port must be > 0".to_string(),
            });
        }

        errors
    }

    /// Check if a config is valid (no errors).
    pub fn is_valid(config: &CortexConfig) -> bool {
        Self::validate(config).is_empty()
    }
}

/// A single diff between two configs.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffEntry {
    pub path: String,
    pub old_value: String,
    pub new_value: String,
}

/// Result of diffing two configs.
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub diffs: Vec<DiffEntry>,
}

impl ConfigDiff {
    /// Diff two configs and produce a migration plan.
    pub fn diff(old: &CortexConfig, new: &CortexConfig) -> Self {
        let mut diffs = Vec::new();

        if old.name != new.name {
            diffs.push(DiffEntry {
                path: "name".to_string(),
                old_value: old.name.clone(),
                new_value: new.name.clone(),
            });
        }
        if old.version != new.version {
            diffs.push(DiffEntry {
                path: "version".to_string(),
                old_value: old.version.clone(),
                new_value: new.version.clone(),
            });
        }

        // Memory diffs
        if old.memory.backend != new.memory.backend {
            diffs.push(DiffEntry {
                path: "memory.backend".to_string(),
                old_value: old.memory.backend.clone(),
                new_value: new.memory.backend.clone(),
            });
        }
        if old.memory.max_entries != new.memory.max_entries {
            diffs.push(DiffEntry {
                path: "memory.max_entries".to_string(),
                old_value: old.memory.max_entries.to_string(),
                new_value: new.memory.max_entries.to_string(),
            });
        }
        if (old.memory.decay_factor - new.memory.decay_factor).abs() > f64::EPSILON {
            diffs.push(DiffEntry {
                path: "memory.decay_factor".to_string(),
                old_value: old.memory.decay_factor.to_string(),
                new_value: new.memory.decay_factor.to_string(),
            });
        }

        // Network diffs
        if old.network.bind != new.network.bind {
            diffs.push(DiffEntry {
                path: "network.bind".to_string(),
                old_value: old.network.bind.clone(),
                new_value: new.network.bind.clone(),
            });
        }
        if old.network.port != new.network.port {
            diffs.push(DiffEntry {
                path: "network.port".to_string(),
                old_value: old.network.port.to_string(),
                new_value: new.network.port.to_string(),
            });
        }
        if old.network.tls != new.network.tls {
            diffs.push(DiffEntry {
                path: "network.tls".to_string(),
                old_value: old.network.tls.to_string(),
                new_value: new.network.tls.to_string(),
            });
        }

        // Agent diffs (added/removed agents)
        for name in old.agents.keys() {
            if !new.agents.contains_key(name) {
                diffs.push(DiffEntry {
                    path: format!("agents.{}", name),
                    old_value: "present".to_string(),
                    new_value: "removed".to_string(),
                });
            }
        }
        for name in new.agents.keys() {
            if !old.agents.contains_key(name) {
                diffs.push(DiffEntry {
                    path: format!("agents.{}", name),
                    old_value: "absent".to_string(),
                    new_value: "added".to_string(),
                });
            }
        }

        Self { diffs }
    }

    /// Number of diffs.
    pub fn len(&self) -> usize {
        self.diffs.len()
    }

    /// Check if there are no diffs.
    pub fn is_empty(&self) -> bool {
        self.diffs.is_empty()
    }

    /// Generate a migration plan as a human-readable string.
    pub fn migration_plan(&self) -> String {
        if self.is_empty() {
            return "No changes detected.".to_string();
        }
        self.diffs
            .iter()
            .map(|d| format!("- {}: {} → {}", d.path, d.old_value, d.new_value))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> CortexConfig {
        let mut agents = HashMap::new();
        agents.insert(
            "main".to_string(),
            AgentConfig {
                agent_type: "chat".to_string(),
                model: "gpt-4".to_string(),
                temperature: 0.7,
            },
        );
        CortexConfig {
            name: "my-exocortex".to_string(),
            version: "0.1.0".to_string(),
            agents,
            memory: MemoryConfig {
                backend: "file".to_string(),
                max_entries: 10000,
                decay_factor: 0.95,
            },
            network: NetworkConfig {
                bind: "127.0.0.1".to_string(),
                port: 8080,
                tls: false,
            },
        }
    }

    #[test]
    fn test_parse_minimal() {
        let toml = r#"
name = "test-project"
"#;
        let config = TomlParser::parse(toml).unwrap();
        assert_eq!(config.name, "test-project");
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_parse_full() {
        let toml = r#"
name = "my-exocortex"
version = "0.1.0"

[agents.main]
type = "chat"
model = "gpt-4"
temperature = 0.7

[memory]
backend = "sqlite"
max_entries = 5000
decay_factor = 0.9

[network]
bind = "0.0.0.0"
port = 3000
tls = true
"#;
        let config = TomlParser::parse(toml).unwrap();
        assert_eq!(config.name, "my-exocortex");
        assert_eq!(config.agents["main"].agent_type, "chat");
        assert_eq!(config.memory.backend, "sqlite");
        assert_eq!(config.network.port, 3000);
        assert!(config.network.tls);
    }

    #[test]
    fn test_parse_invalid() {
        let toml = "not valid toml {{{{";
        assert!(TomlParser::parse(toml).is_err());
    }

    #[test]
    fn test_serialize() {
        let config = sample_config();
        let toml = TomlSerializer::serialize(&config).unwrap();
        assert!(toml.contains("my-exocortex"));
        assert!(toml.contains("chat"));
    }

    #[test]
    fn test_roundtrip() {
        let config = sample_config();
        let toml = TomlSerializer::serialize(&config).unwrap();
        let parsed = TomlParser::parse(&toml).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_validate_valid() {
        let config = sample_config();
        assert!(ConfigValidator::is_valid(&config));
    }

    #[test]
    fn test_validate_empty_name() {
        let mut config = sample_config();
        config.name = "  ".to_string();
        let errors = ConfigValidator::validate(&config);
        assert!(errors.iter().any(|e| e.field == "name"));
    }

    #[test]
    fn test_validate_bad_temperature() {
        let mut config = sample_config();
        config.agents.get_mut("main").unwrap().temperature = 5.0;
        let errors = ConfigValidator::validate(&config);
        assert!(errors.iter().any(|e| e.field == "agents.main.temperature"));
    }

    #[test]
    fn test_validate_zero_max_entries() {
        let mut config = sample_config();
        config.memory.max_entries = 0;
        let errors = ConfigValidator::validate(&config);
        assert!(errors.iter().any(|e| e.field == "memory.max_entries"));
    }

    #[test]
    fn test_validate_bad_port() {
        let mut config = sample_config();
        config.network.port = 0;
        let errors = ConfigValidator::validate(&config);
        assert!(errors.iter().any(|e| e.field == "network.port"));
    }

    #[test]
    fn test_diff_no_changes() {
        let config = sample_config();
        let diff = ConfigDiff::diff(&config, &config);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_name_change() {
        let old = sample_config();
        let mut new = old.clone();
        new.name = "renamed".to_string();
        let diff = ConfigDiff::diff(&old, &new);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff.diffs[0].path, "name");
    }

    #[test]
    fn test_diff_agent_added() {
        let old = sample_config();
        let mut new = old.clone();
        new.agents.insert(
            "helper".to_string(),
            AgentConfig {
                agent_type: "tool".to_string(),
                model: "gpt-3.5".to_string(),
                temperature: 0.5,
            },
        );
        let diff = ConfigDiff::diff(&old, &new);
        assert!(diff.diffs.iter().any(|d| d.path == "agents.helper" && d.new_value == "added"));
    }

    #[test]
    fn test_migration_plan() {
        let old = sample_config();
        let mut new = old.clone();
        new.version = "0.2.0".to_string();
        let diff = ConfigDiff::diff(&old, &new);
        let plan = diff.migration_plan();
        assert!(plan.contains("version"));
    }
}
