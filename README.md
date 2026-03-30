# envguard-rs

Validate environment variables at application startup with a declarative schema.

Define what you expect — types, required/optional, defaults, regex constraints — and `envguard-rs` reads from the environment, validates everything, and returns **all errors at once** instead of crashing on the first missing variable.

## Installation

```toml
[dependencies]
envguard-rs = "0.1"
```

To disable the `regex` dependency (zero dependencies):

```toml
[dependencies]
envguard-rs = { version = "0.1", default-features = false }
```

## Usage

```rust
use envguard_rs::{EnvSchema, VarSpec, EnvType};

fn main() {
    let env = EnvSchema::new()
        .var(VarSpec::new("PORT", EnvType::Int).default("8080"))
        .var(VarSpec::new("API_KEY", EnvType::Str).required())
        .var(
            VarSpec::new("LOG_LEVEL", EnvType::Str)
                .optional()
                .default("info")
                .regex(r"^(debug|info|warn|error)$"),
        )
        .var(VarSpec::new("ENABLE_METRICS", EnvType::Bool).default("false"))
        .validate()
        .unwrap_or_else(|errors| {
            eprintln!("{errors}");
            std::process::exit(1);
        });

    let port = env.get_int("PORT").unwrap();
    let api_key = env.get_str("API_KEY").unwrap();
}
```

If validation fails, all errors are printed at once:

```
Environment validation failed with 2 error(s):
  - 'API_KEY' is required but not set
  - 'LOG_LEVEL' = "verbose" does not match pattern /^(debug|info|warn|error)$/
```

## Variable spec options

| Method | Description |
|--------|-------------|
| `.required()` | Variable must be set (default) |
| `.optional()` | Absence is not an error |
| `.default("value")` | Fallback when variable is absent |
| `.regex(r"pattern")` | Raw string must match the pattern |

## Supported types

| `EnvType` | Rust type | Accepted values |
|-----------|-----------|-----------------|
| `Str` | `String` | Any UTF-8 string |
| `Int` | `i64` | Any integer |
| `Float` | `f64` | Any decimal number |
| `Bool` | `bool` | `true/false`, `1/0`, `yes/no`, `on/off` |

## Accessing values

```rust
env.get_str("VAR")    // Option<&str>
env.get_int("VAR")    // Option<i64>
env.get_float("VAR")  // Option<f64>
env.get_bool("VAR")   // Option<bool>
```

## Features

- `regex-validation` *(enabled by default)*: adds regex constraint support via the [`regex`](https://crates.io/crates/regex) crate. Disable for a zero-dependency build.

## License

MIT
