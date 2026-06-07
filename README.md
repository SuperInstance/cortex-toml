# cortex-toml

> **Configuration-as-code for the Exocortex — parse, validate, serialize, diff, and migrate `.cortex.toml` files**

[![crates.io](https://img.shields.io/crates/v/cortex-toml.svg)](https://crates.io/crates/cortex-toml)
[![docs.rs](https://docs.rs/cortex-toml/badge.svg)](https://docs.rs/cortex-toml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What is Cortex TOML?

Every Exocortex deployment needs configuration — agent definitions, memory settings, network parameters, and more. `cortex-toml` is the library that makes `.cortex.toml` files **first-class configuration artifacts**: typed, validated, versionable, and diffable.

Think of it as the `Cargo.toml` parser for the Exocortex ecosystem. Just as `serde` + `toml` give Rust projects typed configuration, `cortex-toml` gives Exocortex deployments:

- **Parsing**: TOML string → typed `CortexConfig` struct (via serde)
- **Validation**: Check types, ranges, required fields, and cross-field constraints
- **Serialization**: `CortexConfig` → pretty-printed TOML
- **Diffing**: Compare two configs and generate a human-readable migration plan
- **Round-tripping**: Parse → serialize → parse produces identical results

## Why Does This Matter?

Configuration-as-code is a DevOps best practice that brings software engineering rigor to infrastructure:

- **Version control**: Track configuration changes in git, review them in PRs, roll back when needed
- **Validation**: Catch misconfigurations (bad ports, impossible temperatures, missing fields) before deployment
- **Diffing**: See exactly what changed between two config versions — essential for change management
- **Migration planning**: Generate actionable migration steps when upgrading configurations
- **Type safety**: The `CortexConfig` struct is your schema — no silent field name typos or wrong types

Real-world applications:
- **CI/CD pipelines**: Validate `.cortex.toml` in CI before deploying
- **Config drift detection**: Diff running config against committed config to detect unauthorized changes
- **Multi-environment management**: Compare dev/staging/prod configs to ensure consistency
- **Agent onboarding**: New agents get validated configuration from day one

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                  Cortex TOML Pipeline                         │
│                                                              │
│  .cortex.toml                                                │
│  ┌───────────────────────┐                                   │
│  │ name = "my-exocortex" │                                   │
│  │ version = "0.1.0"     │     TomlParser                    │
│  │                       │ ──▶ parse(input) ──▶ CortexConfig │
│  │ [agents.main]         │                                   │
│  │ type = "chat"         │     ┌────────────────────────┐    │
│  │ model = "gpt-4"       │     │    CortexConfig        │    │
│  │                       │     │  ├─ name: String       │    │
│  │ [memory]              │     │  ├─ version: String    │    │
│  │ backend = "sqlite"    │     │  ├─ agents: HashMap    │    │
│  │ max_entries = 5000    │     │  ├─ memory: MemoryCfg  │    │
│  │                       │     │  └─ network: NetworkCfg│    │
│  │ [network]             │     └────────┬───────────────┘    │
│  │ bind = "0.0.0.0"     │              │                     │
│  │ port = 3000          │              ▼                     │
│  │ tls = true           │     ConfigValidator                 │
│  └───────────────────────┘     ┌────────────────────┐        │
│                                │ name non-empty?    │        │
│                                │ temperature 0–2?   │        │
│                                │ port > 0?          │        │
│                                │ decay 0–1?         │        │
│                                └────────┬───────────┘        │
│                                         │                     │
│         TomlSerializer                  ▼                     │
│    serialize(config)──▶ TOML     ConfigDiff                   │
│                                  diff(old, new)──▶ plan       │
│                                  ┌────────────────────┐      │
│                                  │ - port: 8080 → 3000│      │
│                                  │ - agents.x: added  │      │
│                                  └────────────────────┘      │
└──────────────────────────────────────────────────────────────┘
```

## Quick Start

```rust
use cortex_toml::{TomlParser, ConfigValidator, TomlSerializer, CortexConfig};

// Parse a .cortex.toml file
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

let config = TomlParser::parse(toml).expect("Valid TOML");
println!("Project: {} v{}", config.name, config.version);
println!("Agents: {:?}", config.agents.keys().collect::<Vec<_>>());

// Validate the configuration
let errors = ConfigValidator::validate(&config);
if errors.is_empty() {
    println!("✓ Configuration is valid");
} else {
    for err in &errors {
        println!("✗ {}: {}", err.field, err.message);
    }
}
```

### Serialization and Round-Tripping

```rust
// Serialize back to TOML
let output = TomlSerializer::serialize(&config).unwrap();
println!("{}", output);

// Round-trip: parse → serialize → parse should be identical
let reparsed = TomlParser::parse(&output).unwrap();
assert_eq!(config, reparsed);
```

### Diffing Configurations

```rust
use cortex_toml::{ConfigDiff, CortexConfig};

let old_config = TomlParser::parse(r#"
name = "my-exocortex"
version = "0.1.0"
"#).unwrap();

let new_config = TomlParser::parse(r#"
name = "my-exocortex"
version = "0.2.0"

[agents.helper]
type = "tool"
model = "gpt-3.5"
"#).unwrap();

let diff = ConfigDiff::diff(&old_config, &new_config);
if diff.is_empty() {
    println!("No changes");
} else {
    println!("Migration plan:");
    println!("{}", diff.migration_plan());
    // Output:
    // - version: 0.1.0 → 0.2.0
    // - agents.helper: absent → added
}
```

### Validation Error Handling

```rust
use cortex_toml::{TomlParser, ConfigValidator};

let bad_config = TomlParser::parse(r#"
name = "   "

[agents.main]
type = ""
temperature = 5.0
"#).unwrap();

let errors = ConfigValidator::validate(&bad_config);
assert!(!errors.is_empty());
// errors[0]: name is required (whitespace only)
// errors[1]: agents.main.type is required (empty)
// errors[2]: agents.main.temperature must be 0.0-2.0, got 5.0
```

## API Reference

### TomlParser

| Method | Returns | Description |
|--------|---------|-------------|
| `TomlParser::parse(input)` | `Result<CortexConfig, String>` | Parse TOML string to config |

### TomlSerializer

| Method | Returns | Description |
|--------|---------|-------------|
| `TomlSerializer::serialize(config)` | `Result<String, String>` | Serialize config to pretty TOML |

### ConfigValidator

| Method | Returns | Description |
|--------|---------|-------------|
| `ConfigValidator::validate(config)` | `Vec<ValidationError>` | All validation errors |
| `ConfigValidator::is_valid(config)` | `bool` | Quick validity check |

### ConfigDiff

| Method | Returns | Description |
|--------|---------|-------------|
| `ConfigDiff::diff(old, new)` | `ConfigDiff` | Compute differences between configs |
| `diff.len()` | `usize` | Number of changed fields |
| `diff.is_empty()` | `bool` | No changes detected |
| `diff.migration_plan()` | `String` | Human-readable change summary |

### CortexConfig Structure

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | `String` | required | Project name |
| `version` | `String` | `""` | Semantic version |
| `agents` | `HashMap<String, AgentConfig>` | empty | Agent definitions |
| `memory` | `MemoryConfig` | see defaults | Memory subsystem |
| `network` | `NetworkConfig` | see defaults | Network settings |

### AgentConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `agent_type` | `String` | required | Agent type (chat, tool, etc.) |
| `model` | `String` | `""` | Model identifier |
| `temperature` | `f64` | 0.7 | Sampling temperature (0.0–2.0) |

### MemoryConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `backend` | `String` | `"file"` | Storage backend |
| `max_entries` | `u64` | 10000 | Maximum memory entries |
| `decay_factor` | `f64` | 0.95 | Memory decay rate (0.0–1.0) |

### NetworkConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bind` | `String` | `"127.0.0.1"` | Bind address |
| `port` | `u16` | 8080 | Port number |
| `tls` | `bool` | false | Enable TLS |

## Mathematical Background

### Configuration Diffing as Graph Isomorphism

Diffing two configurations is equivalent to computing the **symmetric difference** of two labeled graphs. Each config is a tree (TOML is a tree-structured format), and the diff identifies:
- **Changed leaves**: Scalar values that differ
- **Added subtrees**: New agents or sections present only in the new config
- **Removed subtrees**: Sections present only in the old config

The diff algorithm runs in O(n) time where n is the total number of fields in both configs.

### Validation as Constraint Satisfaction

Each validation rule is a constraint on a field:
```
name:          non-empty ∧ trim(name) ≠ ""
temperature:   0.0 ≤ value ≤ 2.0
max_entries:   value > 0
decay_factor:  0.0 ≤ value ≤ 1.0
port:          0 < value ≤ 65535
```

The validator applies all constraints independently and returns the full set of violations (not just the first one), enabling batch error reporting.

### Decay Factor and Half-Life

The `decay_factor` in memory configuration controls exponential decay:
```
retention(n) = decay_factorⁿ
half_life = −ln(2) / ln(decay_factor)
```

| decay_factor | Half-life (writes) |
|:---:|:---:|
| 0.99 | 69 |
| 0.95 | 13.5 |
| 0.90 | 6.6 |
| 0.85 | 4.3 |

## Installation

```bash
cargo add cortex-toml
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
cortex-toml = "0.1.0"
```

## Related Crates

- [`memory-plimpsest`](https://github.com/SuperInstance/memory-plimpsest) — Layered memory with ghost traces
- [`constellation-map`](https://github.com/SuperInstance/constellation-map) — Fleet visualization as star charts
- [`knowledge-compass`](https://github.com/SuperInstance/knowledge-compass) — Provenance navigation for knowledge graphs
- [`emotional-colorist`](https://github.com/SuperInstance/emotional-colorist) — Valence-based color mapping for agent states

## License

MIT © [SuperInstance](https://github.com/SuperInstance)

---

*Part of the [Exocortex](https://github.com/SuperInstance/exocortex) project — persistent cognitive substrate for multi-agent systems.*
