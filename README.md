# luhtwin
[<img alt="github" src="https://img.shields.io/badge/github-calizoots/luhtwin-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/calizoots/luhtwin)
[<img alt="crates.io" src="https://img.shields.io/crates/v/luhtwin.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/luhtwin)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-luhtwin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/luhtwin)

> Error Handling for Rust

`luhtwin` provides an ergonomic error handling system that emphasizes **context accumulation**, **structured diagnostics**, and **flexible formatting**. Built around the `AnyError` type, it allows you to wrap any error with rich metadata and progressively add context as errors bubble up through your application.

## Core Ideas

- **`AnyError`** — The main error container that wraps any `Error` type with context chains
- **`LuhTwin<T>`** — Type alias for `Result<T, AnyError>`, the primary result type
- **`ErrorContext`** — Structured metadata including messages, file/line info, and custom arguments
- **`Wrap`** — Wrapping existing Results into LuhTwin
- **`Twin`** — Transforming existing Results into LuhTwin
- **`Encase`** — Encase existing LuhTwins in another layer of context

## Quick Start

Add `luhtwin` to your `Cargo.toml`:
```toml
[dependencies]
luhtwin = "0.1.4"
```

## Example
```rust
use luhtwin::{LuhTwin, Twin, Wrap, Encase, at};
use std::fs;

fn read_config_file(path: &str) -> LuhTwin<String> {
    fs::read_to_string(path)
        .wrap(|| format!("failed to read config from {}", path))
}

fn parse_config(content: String) -> LuhTwin<Config> {
    serde_json::from_str(&content)
        .twin()
        .encase(|| "failed to parse config as JSON")
}

fn load_config() -> LuhTwin<Config> {
    let content = read_config_file("config.json")?;
    parse_config(content)
        .encase(|| "config loading failed")
}

fn main() -> LuhTwin<()> {
    let config = load_config()?;
    println!("Config loaded successfully!");
    Ok(())
}
```

## Error Output

When an error occurs, `luhtwin` provides detailed, layered context:
```text
LUHTWIN_FULL=1 to see full errors <3
  1: failed to read config from config.json: No such file or directory
  2: failed to parse config as JSON
  3: config loading failed
source: No such file or directory (os error 2)
backtrace:
disabled backtrace
```

With `LUHTWIN_FULL=1`, you get full context including file/line info and attached arguments.

## Common Patterns

### Converting foreign errors with `.twin()`
```rust
use std::fs::File;

fn open_file() -> LuhTwin<File> {
    File::open("data.txt").twin()
}
```

### Wrapping errors with context using `.wrap()`
```rust
fn read_user_data(id: u32) -> LuhTwin<UserData> {
    read_from_db(id).wrap(|| format!("failed to load user {}", id))
}
```

### Adding layers with `.encase()`
```rust
fn initialize() -> LuhTwin<()> {
    load_config()
        .encase(|| "initialization failed")?;
    connect_db()
        .encase(|| "initialization failed")?;
    Ok(())
}
```

### Using the `at!` macro for rich context
```rust
fn validate_input(input: &str) -> LuhTwin<()> {
    if input.is_empty() {
        return Err(at!("input validation failed")
            .attach("input", input)
            .attach("expected", "non-empty string")
            .into());
    }
    Ok(())
}
```

## Environment Variables

- `LUHTWIN_FULL=1` - Show full error details with all attached arguments
- `RUST_BACKTRACE=1` - Enable backtrace capture (standard Rust behavior)

## Documentation

For full documentation, see [docs.rs/luhtwin](https://docs.rs/luhtwin).

---

> made with love - s.c
