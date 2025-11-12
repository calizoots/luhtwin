# luhtwin

> Horrible Error Handling for Rust

`luhtwin` provides a horrible, non-ergonomic error handling system that emphasizes **context accumulation**, **structured diagnostics**, and **flexible formatting**. Built around the `AnyError` type, it allows you to wrap any error with rich metadata and progressively add context as errors bubble up through your application.

## Core Concepts

- **`AnyError`** — The main error container that wraps any `Error` type with context chains
- **`ErrorContext`** — Structured metadata including messages, file/line info, docs, and severity
- **`AnyErrorBuilder`** — Builder for constructing AnyErrors
- **`LuhTwin<T>`** — Type alias for `Result<T, AnyError>`, the primary result type

## Features

### 🔗 Context Chaining
Add contextual information at each layer of your application:
```rust
use luhtwin::{LuhTwin, at};

fn read_config() -> LuhTwin<String> {
    std::fs::read_to_string("config.toml")
        .map_err(|e| e.into())
        .map_err(|e: luhtwin::AnyError| e.with_context(at!("Failed to read config")))
}
```

### 📝 Error Metadata
Attach documentation links, issue trackers, custom metadata, and severity levels:
```rust
use luhtwin::{anyerror, Level};

let err = anyerror!("Database connection timeout")
    .doc_link("https://docs.example.com/db-errors#timeout")
    .issues(["DB-101", "DB-205"])
    .metadata("host", "localhost:5432")
    .metadata("retry_count", 3)
    .severity(Level::Critical)
    .build();
```

### Multiple Display Formats
Choose the right format for your use case:

- `display_pretty()` — Colorful terminal output
- `display_full()` — Complete diagnostic report with backtrace
- `display_contexts_tree()` — Hierarchical context visualization
- `to_log_format()` — Structured logging format

## Quick Start

### Installation

Add `luhtwin` to your `Cargo.toml`:
```toml
[dependencies]
luhtwin = "0.1.0"
```

### Basic Error Creation
```rust
use luhtwin::{anyerror, at, LuhTwin};

fn might_fail(flag: bool) -> LuhTwin<i32> {
    if flag {
        Ok(42)
    } else {
        Err(anyerror!("Operation failed").build())
    }
}
```

### Adding Context to Existing Errors
```rust
use luhtwin::{Context, LuhTwin};

fn parse_file(path: &str) -> LuhTwin<String> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path))?;
    Ok(content)
}
```

### Working with Context Chains
```rust
use luhtwin::{at, anyerror, LuhTwin};

fn inner() -> LuhTwin<()> {
    Err(anyerror!("Inner error").build())
}

fn middle() -> LuhTwin<()> {
    inner().map_err(|e| e.with_context(at!("Middle layer failed")))
}

fn outer() -> LuhTwin<()> {
    middle().map_err(|e| e.with_context(at!("Outer operation failed")))
}

// Error will contain all three contexts when displayed
```

## Macros

- `at!` — Create an `ErrorContext` at the current file/line
- `anyerror!` — Create an `AnyErrorBuilder`
- `bail!` — Return early with an error
- `ensure!` — Assert a condition or return an error
- `context!` — Add context to a result

## Extension Traits

- `Context` — Add context to any `Result<T, E>` where `E: Error`
- `MapErrExt` — Map errors with additional context
- `LogError` — Convenient error logging methods

## Error Display Examples

### Pretty Display (for terminals)
```text
ERROR error: Failed to connect to database
  --> src/db.rs:45

context chain:
  1. Failed to connect to database
  2. Network timeout occurred

caused by: Connection refused (os error 111)
```

### Tree Display (hierarchical contexts)
```text
└─ Failed to connect to database
    at src/db.rs:45
    doc: https://docs.example.com/db-errors
    issue: DB-101
├─ Network timeout occurred
    at src/network.rs:102
```

### Log Format (structured logging)
```text
message="Failed to connect to database" severity=Critical location="src/db.rs:45" source="Connection refused"
```

---

> made with love s.c - 2025 :3
