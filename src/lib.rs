//! [![github]](https://github.com/calizoots/luhtwin)&ensp;[![crates-io]](https://crates.io/crates/luhtwin)&ensp;[![docs-rs]](https://docs.rs/luhtwin)
//!
//! [github]: https://img.shields.io/badge/github-calizoots/anyhow-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/crates/v/luhtwin.svg?style=for-the-badge&color=fc8d62&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-luhtwin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! # luhtwin - Horrible Error Handling for Rust
//!
//! `luhtwin` provides a horrible, non-ergonomic error handling system that emphasizes
//! **context accumulation**, **structured diagnostics**, and **flexible formatting**.
//! Built around the [`AnyError`] type, it allows you to wrap any error with rich
//! metadata and progressively add context as errors bubble up through your application.
//!
//! ## Core Concepts
//!
//! - **[`AnyError`]** — The main error container that wraps any `Error` type with context chains
//! - **[`ErrorContext`]** — Structured metadata including messages, file/line info, docs, and severity
//! - **[`AnyErrorBuilder`]** — Builder for constructing AnyErrors.
//! - **[`LuhTwin<T>`]** — Type alias for `Result<T, AnyError>`, the primary result type
//!
//! ## Key Features
//!
//! ### Context Chaining
//! Add contextual information at each layer of your application:
//! ```rust
//! use luhtwin::{LuhTwin, at};
//!
//! fn read_config() -> LuhTwin<String> {
//!     std::fs::read_to_string("config.toml")
//!         .map_err(|e| e.into())
//!         .map_err(|e: luhtwin::AnyError| e.with_context(at!("Failed to read config")))
//! }
//! ```
//!
//! ### Error Metadata
//! Attach documentation links, issue trackers, custom metadata, and severity levels:
//! ```rust
//! use luhtwin::{anyerror, Level};
//!
//! let err = anyerror!("Database connection timeout")
//!     .doc_link("https://docs.example.com/db-errors#timeout")
//!     .issues(["DB-101", "DB-205"])
//!     .metadata("host", "localhost:5432")
//!     .metadata("retry_count", 3)
//!     .severity(Level::Critical)
//!     .build();
//! ```
//!
//! ### Multiple Display Formats
//! Choose the right format for your use case:
//! - `display_pretty()` — Colorful terminal output
//! - `display_full()` — Complete diagnostic report with backtrace
//! - `display_contexts_tree()` — Hierarchical context visualization
//! - `to_log_format()` — Structured logging format
//!
//! ## Quick Start
//!
//! ### Basic Error Creation
//! ```rust
//! use luhtwin::{anyerror, at, LuhTwin};
//!
//! fn might_fail(flag: bool) -> LuhTwin<i32> {
//!     if flag {
//!         Ok(42)
//!     } else {
//!         Err(anyerror!("Operation failed").build())
//!     }
//! }
//! ```
//!
//! ### Adding Context to Existing Errors
//! ```rust
//! use luhtwin::{Context, LuhTwin};
//!
//! fn parse_file(path: &str) -> LuhTwin<String> {
//!     let content = std::fs::read_to_string(path)
//!         .context(format!("Failed to read file: {}", path))?;
//!     Ok(content)
//! }
//! ```
//!
//! ### Working with Context Chains
//! ```rust
//! use luhtwin::{at, anyerror, LuhTwin};
//!
//! fn inner() -> LuhTwin<()> {
//!     Err(anyerror!("Inner error").build())
//! }
//!
//! fn middle() -> LuhTwin<()> {
//!     inner().map_err(|e| e.with_context(at!("Middle layer failed")))
//! }
//!
//! fn outer() -> LuhTwin<()> {
//!     middle().map_err(|e| e.with_context(at!("Outer operation failed")))
//! }
//!
//! // Error will contain all three contexts when displayed
//! ```
//!
//! ## Macros
//!
//! - [`at!`] — Create an `ErrorContext` at the current file/line
//! - [`anyerror!`] — Create an `AnyErrorBuilder`
//! - [`bail!`] — Return early with an error
//! - [`ensure!`] — Assert a condition or return an error
//! - [`context!`] — Add context to a result
//!
//! ## Extension Traits
//!
//! - [`Context`] — Add context to any `Result<T, E>` where `E: Error`
//! - [`MapErrExt`] — Map errors with additional context
//! - [`LogError`] — Convenient error logging methods
//!
//! ## Error Display Examples
//!
//! ### Pretty Display (for terminals)
//! ```text
//! ERROR error: Failed to connect to database
//!   --> src/db.rs:45
//!
//! context chain:
//!   1. Failed to connect to database
//!   2. Network timeout occurred
//!
//! caused by: Connection refused (os error 111)
//! ```
//!
//! ### Tree Display (hierarchical contexts)
//! ```text
//! └─ Failed to connect to database
//!     at src/db.rs:45
//!     doc: https://docs.example.com/db-errors
//!     issue: DB-101
//! ├─ Network timeout occurred
//!     at src/network.rs:102
//! ```
//!
//! ### Log Format (structured logging)
//! ```text
//! message="Failed to connect to database" severity=Critical location="src/db.rs:45" source="Connection refused"
//! ```
//!
//! > made with love s.c - 2025 :3

#[cfg(test)]
mod tests;

use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use std::backtrace::Backtrace;
use std::sync::Arc;

pub use luhlog::Level;
pub use luhlog;

/// A boxed error type that is `Send`, `Sync`, and `'static`.
/// Just used for convience and also makes refactoring easier.
pub type ErrorSource = Box<dyn Error + Send + Sync + 'static>;

/// `ErrorContext` is the main structure for representing error metadata in luhtwin.
/// It includes a message, file/line info, documentation links, related issues, metadata,
/// and a severity level.
///
/// # Provided Methods
///
/// - `with_doc_link(link)` — attach a documentation link to the error.
/// - `with_issues(issues)` — attach a list of related issues.
/// - `with_severity(level)` — change the severity of the error.
/// - `with_metadata(key, value)` — attach custom metadata.
///
/// # Examples
/// > mostly you would use this with the `luhtwin::at!` macro
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::{at, Level};
///
/// // Basic usage
/// let err = at!("Something went wrong");
/// println!("{}", err);
/// let warn = at!("This is a warning", Level::Warn)
///     .with_doc_link("https://docs.example.com/warnings")
///     .with_issues(vec!["ISSUE-123", "ISSUE-456"])
///     .with_metadata("user_id", 42);
///
/// println!("{}", warn);
/// let unknown = at!();
/// println!("{}", unknown);
/// ```
/// ---------------------------------------------------------------------------
///
/// # See Also
/// - [`at!`] - for creating an ErrorContext on demand
/// - [`AnyError`] - main structure for storing ErrorContexts
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub msg: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub doc_link: Option<String>,
    pub issues: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub severity: Level,
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = self.severity.to_string();

        writeln!(f, "{}: {}", sev, self.msg)?;

        if let Some(file) = &self.file {
            if let Some(line) = self.line {
                writeln!(f, " --> {}:{}", file, line)?;
            } else {
                writeln!(f, " --> {}", file)?;
            }
        }

        if let Some(link) = &self.doc_link {
            writeln!(f, " documentation: {}", link)?;
        }

        if !self.issues.is_empty() {
            writeln!(f, "\nrelated issues:")?;
            for issue in &self.issues {
                writeln!(f, "    - {}", issue)?;
            }
        }

        if !self.metadata.is_empty() {
            writeln!(f, "\nmetadata:")?;
            // sort keys for stable output
            let mut keys: Vec<_> = self.metadata.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(value) = self.metadata.get(key) {
                    writeln!(f, "    {}: {}", key, value)?;
                }
            }
        }

        Ok(())
    }
}

impl ErrorContext {
    /// Changes the doc link for `luhtwin::ErrorContext`
    pub fn with_doc_link(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }

    /// Changes the issues vector for `luhtwin::ErrorContext`
    pub fn with_issues<I, S>(mut self, issues: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.issues.extend(issues.into_iter().map(Into::into));
        self
    }

    /// Changes the severity for `luhtwin::ErrorContext`
    pub fn with_severity(mut self, severity: Level) -> Self {
        self.severity = severity;
        self
    }

    /// Adds to the metadata of an `luhtwin::ErrorContext`
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: fmt::Display,
    {
        self.metadata.insert(key.into(), value.to_string());
        self
    }
}

/// `AnyError` is the main error type in luhtwin that acts as a **composable, context-rich error container**.
/// 
/// It can wrap any underlying error (`ErrorSource`), attach multiple `ErrorContext`s, and track
/// additional metadata such as backtraces and logging status. Unlike plain `Box<dyn Error>`,
/// `AnyError` provides structured error information, formatted displays, and utility methods
/// for diagnostics, logging, and error chaining.
///
/// # Provided Methods
///
/// - `new(err)` — creates a new `AnyError` from any underlying error.
/// - `with_context(ctx)` — appends an additional `ErrorContext` to the error chain.
/// - `max_severity()` — returns the maximum severity among all contexts.
/// - `to_log_format()` — returns a structured string suitable for logging systems.
/// - `display_pretty()` — ANSI-colored, human-friendly display of the error.
/// - `display_contexts()` — compact linear view of all attached contexts.
/// - `display_contexts_tree()` — hierarchical, tree-like view of the error chain.
/// - `display_backtrace()` — prints the captured backtrace.
/// - `display_full()` — comprehensive report including severity, contexts, root cause, and backtrace.
/// - `root_cause()` — retrieves the underlying root cause of the error.
/// - `iter_sources()` — iterator over the full chain of source errors.
/// - `mark_logged()` / `is_logged()` — mark the error as logged or query its status.
///
/// # See also
///
/// - [`ErrorContext`] — for attaching structured context metadata.
/// - [`at!`] — for ergonomic error context construction.
/// - [`AnyErrorBuilder`] - for ergonomic AnyError construction
pub struct AnyError {
    contexts: Vec<ErrorContext>,
    source: Option<ErrorSource>,
    backtrace: Backtrace,
    logged: std::sync::atomic::AtomicBool,
}

/// A specialized `Result` type that uses `AnyError` as its error variant.
///
/// `LuhTwin<T>` is a convenient alias for `Result<T, luhtwin::AnyError>`
///
/// # Example
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::{LuhTwin, anyerror};
///
/// fn do_something(flag: bool) -> LuhTwin<i32> {
///     if flag {
///         Ok(42)
///     } else {
///         Err(anyerror!("something went wrong").build())
///     }
/// }
///
/// let result = do_something(false);
/// match result {
///     Ok(val) => println!("Success: {}", val),
///     Err(err) => eprintln!("Error: {}", err.display_pretty()),
/// }
/// ```
/// ---------------------------------------------------------------------------
pub type LuhTwin<T> = Result<T, AnyError>;

/// A generic `Result` type that can hold any user-defined error type.
///
/// `BigTwin<T, E>` is simply a `Result<T, E>` alias, used to represent
/// computations that may fail with a custom error `E`. Unlike `LuhTwin`,
/// `BigTwin` does not automatically wrap errors into `AnyError`, allowing
/// the use of domain-specific or third-party error types.
///
/// # Example
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::BigTwin;
///
/// #[derive(Debug)]
/// struct MyError;
///
/// impl std::fmt::Display for MyError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "my custom error")
///     }
/// }
///
/// impl std::error::Error for MyError {}
///
/// fn might_fail(flag: bool) -> BigTwin<i32, MyError> {
///     if flag {
///         Ok(100)
///     } else {
///         Err(MyError)
///     }
/// }
///
/// let result = might_fail(false);
/// if let Err(e) = result {
///     eprintln!("Failed with: {}", e);
/// }
/// ```
/// ---------------------------------------------------------------------------
pub type BigTwin<T, E> = Result<T, E>;

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ctx) = self.contexts.last() {
            write!(f, "{}", ctx.msg)
        } else if let Some(src) = &self.source {
            write!(f, "{}", src)
        } else {
            write!(f, "unknown error")
        }
    }
}

impl fmt::Debug for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n==== error ====")?;

        if !self.contexts.is_empty() {
            writeln!(f, "contexts:")?;
            for (i, ctx) in self.contexts.iter().enumerate() {
                writeln!(f, "  {}: {}", i + 1, ctx.msg)?;
            }
        } else {
            writeln!(f, "no contexts")?;
        }

        if let Some(src) = &self.source {
            writeln!(f, "source: {}", src)?;
        }

        writeln!(f, "backtrace: {}", self.backtrace)?;
        writeln!(f, "logged: {}", self.logged.load(std::sync::atomic::Ordering::Relaxed))?;
        Ok(())
    }
}

impl Error for AnyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn Error + 'static))
    }
}

impl AnyError {
    /// Create a new AnyError
    pub fn new<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            contexts: vec!(),
            source: Some(Box::new(err)),
            backtrace: Backtrace::capture(),
            logged: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Marks the Error as logged
    pub fn mark_logged(&self) {
        self.logged.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Checks if the error is logged
    pub fn is_logged(&self) -> bool {
        self.logged.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct ErrorSourceWrapper(pub Box<dyn CloneableError>);

impl std::fmt::Display for ErrorSourceWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Clone for ErrorSourceWrapper {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

impl Error for ErrorSourceWrapper {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_error())
    }
}

pub trait CloneableError: Error + Send + Sync {
    /// Simple clone
    fn clone_box(&self) -> Box<dyn CloneableError>;
    /// Converts into static error
    fn as_error(&self) -> &(dyn Error + 'static);
    /// Converts into a boxed error 
    fn into_error_box(self: Box<Self>) -> Box<dyn Error + Send + Sync>;
}

#[derive(Debug)]
pub struct NonCloneableWrapper {
    msg: String,
}

impl NonCloneableWrapper {
    pub fn new<E: std::error::Error>(err: &E) -> Self {
        Self { msg: err.to_string() }
    }
}

impl std::fmt::Display for NonCloneableWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for NonCloneableWrapper {}

impl CloneableError for NonCloneableWrapper {
    fn clone_box(&self) -> Box<dyn CloneableError> {
        Box::new(self.clone())
    }

    fn as_error(&self) -> &(dyn std::error::Error + 'static) {
        self
    }

    fn into_error_box(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync> {
        self
    }
}

pub trait ErrorClonableExt: Error + Send + Sync + 'static {
    fn make_clonable(&self) -> Box<dyn CloneableError>;
}

impl<T> ErrorClonableExt for T
where
    T: Error + Send + Sync + 'static,
{
    /// Converts a non clonable error into a cloneable error
    fn make_clonable(&self) -> Box<dyn CloneableError> {
        if let Some(clonable) = (self as &dyn std::any::Any).downcast_ref::<Box<dyn CloneableError>>() {
            return clonable.clone_box();
        }
        Box::new(NonCloneableWrapper::new(self))
    }
}

impl Clone for NonCloneableWrapper {
    fn clone(&self) -> Self {
        Self { msg: self.msg.clone() }
    }
}

/// A builder for creating `AnyError` instances.
///
/// # Example
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::{AnyErrorBuilder, Level};
/// let err = AnyErrorBuilder::new("Failed to load configuration")
///     .doc_link("https://example.com/docs/errors#config")
///     .issues(vec!["CONFIG-101", "CONFIG-102"])
///     .metadata("module", "config_loader")
///     .severity(Level::Critical)
///     .build();
///
/// eprintln!("{}", err.display_pretty());
/// ```
/// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct AnyErrorBuilder {
    ctx: ErrorContext,
    source: Option<Arc<ErrorSourceWrapper>>,
}

impl AnyErrorBuilder {
    /// Creates a new `AnyErrorBuilder` with a main error message.
    ///
    /// Automatically captures the current file and line number, and sets
    /// the default severity to `Level::Error`.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            ctx: ErrorContext {
                msg: msg.into(),
                file: Some(file!().to_string()),
                line: Some(line!()),
                doc_link: None,
                issues: vec![],
                metadata: HashMap::new(),
                severity: Level::Error,
            },
            source: None,
        }
    }

    /// Adds a documentation link to the error context.
    pub fn doc_link(mut self, link: impl Into<String>) -> Self {
        self.ctx.doc_link = Some(link.into());
        self
    }

    /// Attaches a list of issue identifiers related to this error.
    pub fn issues<I, S>(mut self, issues: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.ctx.issues.extend(issues.into_iter().map(Into::into));
        self
    }

    /// Adds arbitrary metadata as key-value pairs to the error context.
    pub fn metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: fmt::Display,
    {
        self.ctx.metadata.insert(key.into(), value.to_string());
        self
    }

    /// Sets the severity level of the error.
    pub fn severity(mut self, severity: Level) -> Self {
        self.ctx.severity = severity;
        self
    }

    /// Attaches a source error to this error.
    pub fn source<E>(mut self, err: E) -> Self
    where
        E:  Error + Send + Sync + 'static,
    {
        self.source = Some(Arc::new(ErrorSourceWrapper(err.make_clonable())));
        self
    }

    /// Finalizes the builder and returns an `AnyError`.
    pub fn build(self) -> AnyError {
        AnyError {
            contexts: vec![self.ctx],
            source: self.source.map(|arc_err| {
                let wrapper: Box<dyn CloneableError> = match Arc::try_unwrap(arc_err) {
                    Ok(wrapper) => wrapper.0,
                    Err(shared) => shared.0.clone_box(),
                };
                wrapper.into_error_box()
            }),
            backtrace: Backtrace::capture(),
            logged: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl<E> From<E> for AnyError
where
    E: Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        AnyError::new(err)
    }
}

impl AnyError {
    /// Pushes a contexts to the `AnyError::contexts` vector
    pub fn with_context(mut self, ctx: ErrorContext) -> Self {
        self.contexts.push(ctx);
        self
    }

    /// Returns the max severity in the `AnyError::contexts` vector
    pub fn max_severity(&self) -> Level {
        self.contexts
            .iter()
            .map(|c| c.severity)
            .max()
            .unwrap_or(Level::Error)
    }

    /// Returns the error in a compact log-friendly format.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// # use luhtwin::AnyErrorBuilder;
    /// let err = AnyErrorBuilder::new("Something went wrong").build();
    /// println!("{}", err.to_log_format());
    /// // message="Something went wrong" severity=Error location="src/main.rs:10"
    /// ```
    /// ---------------------------------------------------------------------------
    pub fn to_log_format(&self) -> String {
        let mut parts = vec![];
        
        if let Some(ctx) = self.contexts.last() {
            parts.push(format!("message=\"{}\"", ctx.msg.replace('"', "\\\"")));
        }
        
        parts.push(format!("severity={}", self.max_severity()));
        
        if let Some(ctx) = self.contexts.last() {
            if let Some(file) = &ctx.file {
                if let Some(line) = ctx.line {
                    parts.push(format!("location=\"{}:{}\"", file, line));
                }
            }
        }
        
        if let Some(src) = &self.source {
            parts.push(format!("source=\"{}\"", src.to_string().replace('"', "\\\"")));
        }
        
        parts.join(" ")
    }

    /// Returns a colorful, human-readable representation of the error.
    ///
    /// Uses ANSI color codes to highlight severity.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// # use luhtwin::AnyErrorBuilder;
    /// let err = AnyErrorBuilder::new("Failed to connect").build();
    /// println!("{}", err.display_pretty());
    /// // ERROR error: Failed to connect
    /// //   --> src/main.rs:12
    /// ```
    /// ---------------------------------------------------------------------------
    pub fn display_pretty(&self) -> String {
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const BLUE: &str = "\x1b[34m";
        const GRAY: &str = "\x1b[90m";
        const RESET: &str = "\x1b[0m";
        const BOLD: &str = "\x1b[1m";

        let mut result = String::new();
        
        if let Some(ctx) = self.contexts.last() {
            let color = match ctx.severity {
                Level::Critical | Level::Error => RED,
                Level::Warn => YELLOW,
                Level::Info => BLUE,
                Level::Debug => GRAY,
                Level::Trace => BOLD,
            };
            
            result.push_str(&format!("{}{}error:{} {}{}\n", 
                BOLD, color, RESET, BOLD, ctx.msg));
            result.push_str(RESET);
            
            if let Some(file) = &ctx.file {
                if let Some(line) = ctx.line {
                    result.push_str(&format!("  {}{}-->{} {}:{}\n", 
                        BOLD, BLUE, RESET, file, line));
                }
            }
        }
        
        if self.contexts.len() > 1 {
            result.push_str(&format!("\n{}context chain:{}\n", BOLD, RESET));
            // show from outermost to innermost (reverse order)
            for (i, ctx) in self.contexts.iter().rev().enumerate() {
                result.push_str(&format!("  {}. {}\n", i + 1, ctx.msg));
            }
        }
        
        if let Some(src) = &self.source {
            result.push_str(&format!("\n{}caused by:{} {}\n", BOLD, RESET, src));
        }
        
        result
    }


    /// Returns the contexts in a simple, plain-text format.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// # use luhtwin::AnyErrorBuilder;
    /// let err = AnyErrorBuilder::new("Top level error")
    ///     .doc_link("https://example.com/docs")
    ///     .issues(vec!["ISSUE-101"])
    ///     .build();
    ///
    /// println!("{}", err.display_contexts());
    /// // 1: Top level error
    /// //     [doc: https://example.com/docs]
    /// //     [issues: ISSUE-101]
    /// ```
    /// ---------------------------------------------------------------------------
    pub fn display_contexts(&self) -> String {
        let mut result = String::new();

        for (i, ctx) in self.contexts.iter().rev().enumerate() {
            let mut line = format!("{}: {}", i + 1, ctx.msg);
            if let Some(file) = &ctx.file {
                if let Some(line_num) = ctx.line {
                    line.push_str(&format!(" ({}:{})", file, line_num));
                } else {
                    line.push_str(&format!(" ({})", file));
                }
            }
            result.push_str(&line);

            if let Some(link) = &ctx.doc_link {
                result.push_str(&format!("\n    [doc: {}]", link));
            }

            if !ctx.issues.is_empty() {
                result.push_str(&format!("\n    [issues: {}]", ctx.issues.join(", ")));
            }

            if i != self.contexts.len() - 1 {
                result.push_str("\n-> ");
            }
        }

        result
    }

    /// Returns the contexts as a tree structure.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// use luhtwin::{AnyErrorBuilder, at};
    /// let err = AnyErrorBuilder::new("Outer error")
    ///     .build()
    ///     .with_context(at!("inner error"));
    ///
    /// println!("{}", err.display_contexts_tree());
    /// // └─ Outer error
    /// //     at src/main.rs:10
    /// // ├─ Inner error
    /// //     at src/main.rs:9
    /// ```
    /// ---------------------------------------------------------------------------
    pub fn display_contexts_tree(&self) -> String {
        let mut result = String::new();
        let last_idx = self.contexts.len().saturating_sub(1);

        for (i, ctx) in self.contexts.iter().rev().enumerate() {
            let is_last = i == last_idx;
            let prefix = if is_last { "└─ " } else { "├─ " };
            let child_prefix = if is_last { "    " } else { "│   " };

            result.push_str(&format!("{}{}\n", prefix, ctx.msg));

            if let Some(file) = &ctx.file {
                if let Some(line) = ctx.line {
                    result.push_str(&format!("{}at {}:{}\n", child_prefix, file, line));
                } else {
                    result.push_str(&format!("{}at {}\n", child_prefix, file));
                }
            }

            if let Some(link) = &ctx.doc_link {
                result.push_str(&format!("{}doc: {}\n", child_prefix, link));
            }

            for issue in &ctx.issues {
                result.push_str(&format!("{}issue: {}\n", child_prefix, issue));
            }
        }

        result
    }

    /// Prints out the backtrace will say disabled if RUST_BACKTRACE != 1
    pub fn display_backtrace(&self) -> String {
        format!("{}", self.backtrace)
    }

    /// Displays the full error report, including contexts, source errors, and backtrace.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// # use luhtwin::AnyErrorBuilder;
    /// let root = AnyErrorBuilder::new("Root failure").build();
    /// let err = AnyErrorBuilder::new("Higher-level failure")
    ///     .source(root)
    ///     .build();
    ///
    /// println!("{}", err.display_full());
    /// // === error report ===
    /// // severity: Error
    /// // message: Higher-level failure
    /// //
    /// // context chain:
    /// // └─ Higher-level failure
    /// //     at src/main.rs:10
    /// //
    /// // root cause: Root failure
    /// //
    /// // error chain:
    /// //  0: Root failure
    /// //
    /// // backtrace:
    /// // <backtrace output>
    /// ```
    /// ---------------------------------------------------------------------------
    pub fn display_full(&self) -> String {
        let mut result = String::new();
        
        result.push_str("=== error report ===\n\n");
        result.push_str(&format!("severity: {}\n", self.max_severity()));
        result.push_str(&format!("message: {}\n\n", self));
        
        if !self.contexts.is_empty() {
            result.push_str("context chain:\n");
            result.push_str(&self.display_contexts_tree());
            result.push_str("\n");
        }
        
        if let Some(src) = &self.source {
            result.push_str(&format!("root cause: {}\n\n", src));
            
            // Show full error chain
            result.push_str("error chain:\n");
            for (i, err) in self.iter_sources().enumerate() {
                result.push_str(&format!("  {}: {}\n", i, err));
            }
            result.push_str("\n");
        }
        
        result.push_str("backtrace:\n");
        result.push_str(&self.display_backtrace());
        
        result
    }
}

impl AnyError {
    /// Returns back the root cause
    pub fn root_cause(&self) -> &(dyn Error + 'static) {
        let mut source: &(dyn Error + 'static) = self;
        while let Some(s) = source.source() {
            source = s;
        }
        source
    }

    /// Iterate over all the contexts in an AnyError
    pub fn iter_sources(&self) -> impl Iterator<Item = &(dyn Error + 'static)> + '_ {
        std::iter::successors(Some(self as &(dyn Error + 'static)), |&e| e.source())
    }
}

impl From<ErrorContext> for AnyError {
    /// Converts an `ErrorContext` into an `AnyError`.
    ///
    /// This allows you to easily promote a context into a full error.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// use luhtwin::{at, AnyError};
    /// 
    /// let ctx = at!("Something went wrong")
    ///     .with_doc_link("https://example.com/docs")
    ///     .with_severity(luhtwin::Level::Critical);
    /// 
    /// let err: AnyError = ctx.into();
    /// println!("{}", err.display_pretty());
    /// ```
    /// ---------------------------------------------------------------------------
    fn from(ctx: ErrorContext) -> Self {
        AnyError {
            contexts: vec![ctx],
            source: None,
            backtrace: Backtrace::capture(),
            logged: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl ErrorContext {
    /// Converts this context into a standalone `AnyError`.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```rust
    /// use luhtwin::at;
    /// 
    /// let err = at!("Operation failed")
    ///     .with_doc_link("https://docs.example.com/errors")
    ///     .into_error();
    /// 
    /// println!("{}", err.display_pretty());
    /// ```
    /// ---------------------------------------------------------------------------
    pub fn into_error(self) -> AnyError {
        self.into()
    }
}

/// A extension trait which allows context to be added to any errors/
/// and return back a `luhtwin::LuhTwin` for more error handling!!!
pub trait Context<T> {
    /// Add a context to an arbitrary error type and return back a `luhtwin::LuhTwin`
    fn context<C>(self, msg: C) -> LuhTwin<T> 
    where
        C: fmt::Display;

    /// Add a formatted context to an arbitrary error type and return back a `luhtwin::LuhTwin`
    fn with_context<C, F>(self, f: F) -> LuhTwin<T>
    where
        C: fmt::Display,
        F: FnOnce() -> C;
}

impl<T, E> Context<T> for BigTwin<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn context<C>(self, msg: C) -> LuhTwin<T>
    where
        C: fmt::Display,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!(msg)))
    }

    fn with_context<C, F>(self, f: F) -> LuhTwin<T>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!(f())))
    }
}

/// Extension trait for adding contextual information to errors by mapping them.
pub trait WrapErrExt<T> {
    /// Maps the error and adds additional context, returning a `LuhTwin<T>`.
    ///
    /// # Example
    /// ---------------------------------------------------------------------------
    /// ```ignore
    /// use luhtwin::{BigTwin, MapErrExt, LuhTwin};
    /// fn might_fail() -> BigTwin<i32, std::io::Error> { unimplemented!() }
    ///
    /// let result: LuhTwin<i32> = might_fail()
    ///     .wrap_err_context(|| "Failed during file read");
    ///
    /// // if an error occurs it will be wrapped with a context:
    /// // AnyError { contexts: [ "Failed during file read: <original error>" ], ... }
    /// ```
    /// ---------------------------------------------------------------------------
    fn wrap<F, C>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce() -> C,
        C: fmt::Display;
}

impl<T, E> WrapErrExt<T> for BigTwin<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn wrap<F, C>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce() -> C,
        C: fmt::Display,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", f(), e);
            AnyError::new(e).with_context(at!(msg))
        })
    }
}

/// Extension trait for logging errors conveniently.
pub trait LogError<T> {
    /// Logs the error using the default logging mechanism.
    fn log_error(self) -> Self;
    
    /// Logs the error with a prefix message
    fn log_error_with(self, prefix: &str) -> Self;
    
    /// Logs and then marks it as logged so will not be logged again unless explicitly
    fn log_once(self) -> Self;
}

impl<T> LogError<T> for LuhTwin<T> {
    fn log_error(self) -> Self {
        if let Err(ref e) = self {
            eprintln!("{}", e.display_pretty());
        }
        self
    }
    
    fn log_error_with(self, prefix: &str) -> Self {
        if let Err(ref e) = self {
            eprintln!("{}: {}", prefix, e.display_pretty());
        }
        self
    }
    
    fn log_once(self) -> Self {
        if let Err(ref e) = self {
            if !e.is_logged() {
                eprintln!("{}", e.display_pretty());
                e.mark_logged();
            }
        }
        self
    }
}

/// Creates a new `AnyErrorBuilder` with the given message.
///
/// This macro is a shorthand for `AnyErrorBuilder::new`.
///
/// # Example
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::anyerror;
/// let err = anyerror!("Something went wrong")
///     .doc_link("https://docs.example.com/error")
///     .build();
/// ```
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::anyerror;
/// let code = 404;
/// let err = anyerror!("Request failed with code {}", code)
///     .doc_link("https://docs.example.com/error")
///     .build();
/// ```
#[macro_export]
macro_rules! anyerror {
    ($msg:expr) => {
        $crate::AnyErrorBuilder::new($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::AnyErrorBuilder::new(format!($fmt, $($arg)*))
    };
}

/// Creates a new `ErrorContext` at the current file and line.
///
/// Can optionally specify a severity level. If no severity is provided,
/// defaults to `Level::Error`.
///
/// # Examples
/// ---------------------------------------------------------------------------
/// Basic usage:
/// ```rust
/// use luhtwin::at;
/// let ctx = at!("A simple error context");
/// ```
///
/// ```rust
/// use luhtwin::at;
/// let user_id = 42;
/// let ctx = at!("User {} not found", user_id);
/// ```
///
/// ```rust
/// use luhtwin::{at, luhlog::Level};
/// let ctx = at!("A warning context"; Level::Warn);
/// ```
///
/// ```rust
/// use luhtwin::{at, luhlog::Level};
/// let count = 5;
/// let ctx = at!("Found {} issues", count; Level::Warn);
/// ```
///
/// ```rust
/// use luhtwin::at;
/// let ctx = at!();
/// assert_eq!(ctx.msg, "unknown error");
/// ```
#[macro_export]
macro_rules! at {
    () => {
        $crate::ErrorContext {
            msg: "unknown error".to_string(),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $crate::luhlog::Level::Error,
            metadata: std::collections::HashMap::new(),
        }
    };
    ($fmt:literal, $($arg:expr),+ ; $severity:expr $(,)?) => {
        $crate::ErrorContext {
            msg: format!($fmt, $($arg),+),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $severity,
            metadata: std::collections::HashMap::new(),
        }
    };
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::ErrorContext {
            msg: format!($fmt, $($arg),+),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $crate::luhlog::Level::Error,
            metadata: std::collections::HashMap::new(),
        }
    };
    ($msg:expr ; $severity:expr) => {
        $crate::ErrorContext {
            msg: $msg.to_string(),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $severity,
            metadata: std::collections::HashMap::new(),
        }
    };
    ($msg:expr) => {
        $crate::ErrorContext {
            msg: $msg.to_string(),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $crate::luhlog::Level::Error,
            metadata: std::collections::HashMap::new(),
        }
    };
}

/// Immediately returns an `Err(AnyError)` from a function.
///
/// Accepts formatting arguments like `format!` and converts them into an
/// `AnyError` with a generic `std::io::Error` as the source.
///
/// # Example
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::{bail, LuhTwin};
/// fn do_something() -> LuhTwin<()> {
///     bail!("This operation failed with code {}", 42);
/// }
/// ```
/// ---------------------------------------------------------------------------
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::AnyError::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!($($arg)*),
        )))
    };
}

/// Ensures a condition is true, otherwise returns an error.
///
/// This is a shorthand for:
/// ```ignore
/// if !$cond {
///     bail!(...);
/// }
/// ```
///
/// # Example
/// ---------------------------------------------------------------------------
/// ```rust
/// use luhtwin::{ensure, LuhTwin};
/// fn check_value(x: i32) -> LuhTwin<()> {
///     ensure!(x > 0, "x must be positive, got {}", x);
///     Ok(())
/// }
/// ```
/// ---------------------------------------------------------------------------
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::bail!($($arg)*);
        }
    };
}

/// Adds a context to a result or error-like type.
#[macro_export]
macro_rules! context {
    ($res:expr, $msg:expr) => {
        $res.with_context(|| $msg)
    };
}
