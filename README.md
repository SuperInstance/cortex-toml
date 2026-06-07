# cortex-toml

> **Configuration-as-code for the Exocortex**

[![crates.io](https://img.shields.io/crates/v/cortex-toml.svg)](https://crates.io/crates/cortex-toml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Parses and validates .cortex.toml configuration files for the Exocortex. Defines the full configuration schema including memory tiers, shadow pipeline settings, dream cycle parameters, and fleet coordination options.

## Example .cortex.toml

```toml
[cortex]
name = "my-cortex"
version = "0.1.0"

[memory]
tiers = ["episodic", "semantic", "procedural"]
decay_half_life_days = 30.0

[shadows]
layers = ["raw", "structured", "narrative"]
compression_ratio = 0.1

[dream]
cycle_minutes = 30
rem_duration_minutes = 5
```

## Features

- **Parse**: From TOML string to typed Rust struct
- **Serialize**: Back from struct to TOML
- **Validate**: Type checking, range validation, required fields
- **Diff**: Compare two configs and generate migration plans

## Installation

```toml
[dependencies]
cortex-toml = "0.1.0"
```

## License

MIT © [SuperInstance](https://github.com/SuperInstance)

---

*Part of the [Exocortex](https://github.com/SuperInstance/exocortex) project — persistent cognitive substrate for multi-agent systems.*
