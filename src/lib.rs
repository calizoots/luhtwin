//! [![github]](https://github.com/calizoots/luhtwin)&ensp;[![crates-io]](https://crates.io/crates/luhtwin)&ensp;[![docs-rs]](https://docs.rs/luhtwin)
//!
//! [github]: https://img.shields.io/badge/github-calizoots/anyhow-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/crates/v/luhtwin.svg?style=for-the-badge&color=fc8d62&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-luhtwin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//! # luhtwin - Horrible Error Handling for Rust
//!
//! `luhtwin` provides a ergonomic error handling system that emphasizes
//! **context accumulation**, **structured diagnostics**, and **flexible formatting**.
//! Built around the [`AnyError`] type, it allows you to wrap any error with rich
//! metadata and progressively add context as errors bubble up through your application.
//!
//! ## Core Ideas
//!
//! - **[`AnyError`]** — The main error container that wraps any `Error` type with context chains
//! - **[`LuhTwin<T>`]** — Type alias for `Result<T, AnyError>`, the primary result type
//! - **[`ErrorContext`]** — Structured metadata including messages, file/line info, docs, and severity
//! - **[`Wrap`]** — Wrapping existing Results into LuhTwin.
//! - **[`Twin`]** — Transforming existing Results into LuhTwin.
//! - **[`Encase`]** — Encase existing LuhTwins in another layer of context.
//!
//! ## Getting Started
//!
//! Add `luhtwin` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! luhtwin = "0.1.4"
//! ```
//!
//! ## Quick Example
//!
//! ```ignore
//! use luhtwin::{LuhTwin, Twin, Wrap, Encase, at};
//! use std::fs;
//! use std::io;
//!
//! fn read_config_file(path: &str) -> LuhTwin<String> {
//!     fs::read_to_string(path)
//!         .wrap(|| format!("failed to read config from {}", path))
//! }
//!
//! fn parse_config(content: String) -> LuhTwin<Config> {
//!     serde_json::from_str(&content)
//!         .twin()
//!         .encase(|| "failed to parse config as JSON")
//! }
//!
//! fn load_config() -> LuhTwin<Config> {
//!     let content = read_config_file("config.json")?;
//!     parse_config(content)
//!         .encase(|| "config loading failed")
//! }
//!
//! fn main() -> LuhTwin<()> {
//!     let config = load_config()?;
//!     println!("Config loaded successfully!");
//!     Ok(())
//! }
//! ```
//!
//! ## Error Output
//!
//! When an error occurs, `luhtwin` provides detailed, layered context:
//!
//! ```text
//! LUHTWIN_FULL=1 to see full errors <3
//!   1: failed to read config from config.json: No such file or directory
//!   2: failed to parse config as JSON
//!   3: config loading failed
//! source: No such file or directory (os error 2)
//! backtrace:
//! disabled backtrace
//! ```
//!
//! With `LUHTWIN_FULL=1`, you get the full context including file/line info:
//!
//! ```text
//!   1: failed to read config from config.json: No such file or directory
//!       - file: "src/main.rs"
//!       - line: 8
//!
//!   2: failed to parse config as JSON
//!       - file: "src/main.rs"
//!       - line: 14
//!
//!   3: config loading failed
//!       - file: "src/main.rs"
//!       - line: 19
//! ```
//!
//! ## Common Patterns
//!
//! ### Converting foreign errors with `.twin()`
//!
//! ```ignore
//! use std::fs::File;
//!
//! fn open_file() -> LuhTwin<File> {
//!     File::open("data.txt").twin()
//! }
//! ```
//!
//! ### Wrapping errors with context using `.wrap()`
//!
//! ```ignore
//! fn read_user_data(id: u32) -> LuhTwin<UserData> {
//!     read_from_db(id).wrap(|| format!("failed to load user {}", id))
//! }
//! ```
//!
//! ### Adding layers with `.encase()`
//!
//! ```ignore
//! fn initialize() -> LuhTwin<()> {
//!     load_config()
//!         .encase(|| "initialization failed")?;
//!     connect_db()
//!         .encase(|| "initialization failed")?;
//!     Ok(())
//! }
//! ```
//!
//! ### Using the `at!` macro for rich context
//!
//! ```ignore
//! fn validate_input(input: &str) -> LuhTwin<()> {
//!     if input.is_empty() {
//!         return Err(at!("input validation failed")
//!             .attach("input", input)
//!             .attach("expected", "non-empty string")
//!             .into());
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Environment Variables
//!
//! - `LUHTWIN_FULL=1` - Show full error details with all attached arguments
//! - `RUST_BACKTRACE=1` - Enable backtrace capture (standard Rust behavior)
//!

#[cfg(test)]
mod tests;

use std::backtrace::Backtrace;
use std::env;
use std::error::Error;
use std::any::Any;
use std::fmt::{self, Debug};

/// Just a base trait ease of refactoring <3
pub type ErrorSource = Box<dyn Error + Send + Sync + 'static>;

/// # ErrorArg
///
/// All arguments passed to an `luhtwin::ErrorContext` must provide these 3 things
///
/// `fn name(&self) -> &str` -> returns the name of the value
/// `fn display(&self) -> String` -> returns a string which corresponds to output in console
/// `fn as_any(&self) -> &dyn Any` -> returns an trait object Any or in other words a borrow to a
/// shared reference can be used to dereference to get original value in code if needs really be
pub trait ErrorArg: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn display(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

/// # Arg
///
/// The default implementation of `luhtwin::ErrorArg` (you can write your own)
///
/// ## note
///
/// `Arg::display()` always returns the debug display of your value
#[derive(Debug)]
pub struct Arg<T: Debug + Send + Sync + 'static> {
    pub name: String,
    pub value: T,
}

impl<T: Debug + Send + Sync + 'static> ErrorArg for Arg<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn display(&self) -> String {
        format!("{:?}", self.value)
    }

    fn as_any(&self) -> &dyn Any {
        &self.value
    }
}

/// # ErrorContext
/// 
/// The main context type for `luhtwin::AnyError` fairly simple
/// contains a message and a list of Arguments which all follow a trait
/// so you can roll your own. See `luhtwin::Arg` and `luhtwin::ErrorArg` trait
///
/// ## Provided Methods
///
/// ## `fn new(msg: impl Into<String>) -> ErrorContext`
/// 
/// Returns a new ErrorContext with provided message and empty arguments
///
/// ## `fn attach_other<E: ErrorArg + 'static>(mut self, arg: E) -> ErrorContext`
/// 
/// Pushes a new argument (implementing `luhtwin::ErrorArg`) to self.args and returns itself
///
/// ## `fn attach<T: Debug + Send + Sync + 'static>(mut self, name: impl Into<String>, value: T) -> ErrorContext`
/// 
/// Pushes a new argument with the given name with an unspecified type and returns itself
///
/// ## `fn file(mut self, file: impl Into<String>) -> Self`
/// 
/// Adds an argument with a predefined name "file"
///
/// ## `fn line(mut self, line: u32) -> Self`
/// 
/// Adds an argument with a predefined name "line"
///
/// ## `fn into_error(self) -> AnyError`
///
/// Calls into on itself useful for when you need a specified type 
///
/// ## Examples
///
/// The main way to interface with an `luhtwin::ErrorContext` is with the `luhtwin::at` macro
/// here is a basic example of that but do read the at macro documentation
///
/// ```ignore
/// fn main() -> LuhTwin<()> {
///     Err(at!("oopsie happened")
///         .attach("issues", vec!["only", "possible", "issue"]).into()) 
/// }
/// ```
///
/// Or you could construct one manually
///
/// ```ignore
/// let ctx = ErrorContext::new("oops oh no happened")
///     .file(file!())
///     .line(line!())
///     .attach("issues", vec!["#12042", "#12079"])
/// ```
pub struct ErrorContext {
    pub message: String,
    pub args: Vec<Box<dyn ErrorArg>>,
}

impl ErrorContext {
    /// Returns a new ErrorContext with provided message and empty arguments
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            args: vec![]
        }
    }

    /// Pushes a new argument (implementing `luhtwin::ErrorArg`) to self.args and returns itself
    pub fn attach_other<E: ErrorArg + 'static>(mut self, arg: E) -> Self {
        self.args.push(Box::new(arg));
        self
    }

    /// Pushes a new argument with the given name with an unspecified type and returns itself
    pub fn attach<T: Debug + Send + Sync + 'static>(
        mut self,
        name: impl Into<String>,
        value: T,
    ) -> Self {
        self.args.push(Box::new(Arg {
            name: name.into(),
            value
        }));
        self
    }

    /// Adds an argument with a predefined name "file"
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.args.push(Box::new(Arg {
            name: "file".to_string(),
            value: file.into()
        }));
        self
    }

    /// Adds and argument with a predefined name "line"
    pub fn line(mut self, line: u32) -> Self {
        self.args.push(Box::new(Arg {
            name: "line".to_string(),
            value: line
        }));
        self
    }

    /// Calls into on itself useful for when you need a specified type 
    pub fn into_error(self) -> AnyError {
        self.into()
    }
}

// easy convertions from string types to ErrorContext

impl From<&str> for ErrorContext {
    /// just calls `luhtwin::at` macro
    fn from(str: &str) -> Self {
       at!(str) 
    }
}

impl From<String> for ErrorContext {
    /// just calls `luhtwin::at` macro
    fn from(str: String) -> Self {
       at!(str) 
    }
}

impl std::fmt::Display for ErrorContext {
    /*
    example of how this looks:
    
    process is already running pid file at /tmp/luhproc/mothapp-3lk5g1qvq1ub/pid
        - file: "/Users/crack/Bang/luhproc/src/lib.rs"
        - line: 191
     */

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;

        for arg in &self.args {
            writeln!(f, "    - {}: {}", arg.name(), arg.display())?;
        }

        Ok(())
    }
}

/// # AnyError
///
/// The main error type in this library used in `luhtwin:LuhTwin` which is a
/// Result<T, AnyError>... AnyError provides a lot of flexibility with a list
/// of ErrorContext(s) also provides a Backtrace and an optional source see `luhtwin::ErrorSource`
///
/// Rarely will you interface with this type by itself rather in predefined abstractions
/// given to you see `luhtwin::Wrap`, `luhtwin::Encase`, `luhtwin::Twin` and `luhtwin::Context`
///
/// ## Provided Methods
///
/// ## `fn new<E>(err: E) -> AnyError where E: Error + Send + Sync + 'static`
///
/// Takes all the error types you could want and turn them into an AnyError with an empty context
/// will capture a backtrace if enabled through RUST_BACKTRACE=1
///
/// ## `fn with_context(mut self, ctx: ErrorContext) -> AnyError`
///
/// Appends the given context to self.contexts
///
/// ## `fn root_cause(&self) -> &(dyn Error + 'static)`
///
/// Returns the first error in self.contexts in other words the root cause
///
/// ## `fn iter_sources(&self) -> impl Iterator<Item = &(dyn Error + 'static)> + '_ `
///
/// This is just remnants from the last version kept here for ease of use don't use though
/// its just dumb lmao
///
/// ## Examples
///
/// As I said beforehand you would rarely use this by itself you would use it like this
///
/// ```ignore
/// fn this_fails() -> LuhTwin<()> {
///     Err(io::Error::new(io::ErrorKind::Other, "file missing")
///             .twin())
/// }
/// fn main() -> LuhTwin<()> {
///    this_fails().encase(|| "it failed like expected")?;
///    Ok(())
/// }
/// ```
///
/// Or wrap pre-existing errors
///
/// ```ignore
/// fn this_fails() -> Result<(), io::Error> {
///     Err(io::Error::new(io::ErrorKind::Other, "idk what happened"))
/// }
/// fn main() -> LuhTwin<()> {
///     this_fails().wrap(|| "it failed")?;
///     Ok(())
/// }
/// ```
pub struct AnyError {
    contexts: Vec<ErrorContext>,
    source: Option<ErrorSource>,
    backtrace: Backtrace,
}

impl AnyError {
    /// Takes all the error types you could want and turn them into an AnyError with an empty context
    /// will capture a backtrace if enabled through RUST_BACKTRACE=1
    pub fn new<E>(err: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            contexts: vec!(),
            source: Some(Box::new(err)),
            backtrace: Backtrace::capture(),
        }
    }

    /// Appends the given context to self.contexts
    pub fn with_context(mut self, ctx: ErrorContext) -> Self {
        self.contexts.push(ctx);
        self
    }

    /// Returns the first error in self.contexts in other words the root cause
    pub fn root_cause(&self) -> &(dyn Error + 'static) {
        let mut source: &(dyn Error + 'static) = self;
        while let Some(s) = source.source() {
            source = s;
        }
        source
    }

    /// This is just remnants from the last version kept here for ease of use don't use though
    /// its just dumb lmao
    pub fn iter_sources(&self) -> impl Iterator<Item = &(dyn Error + 'static)> + '_ {
        std::iter::successors(Some(self as &(dyn Error + 'static)), |&e| e.source())
    }
}

impl From<ErrorContext> for AnyError {
    /// Promotes a ErrorContext to an AnyError with an empty source and one (the given) context
    fn from(ctx: ErrorContext) -> Self {
        AnyError {
            contexts: vec![ctx],
            source: None,
            backtrace: Backtrace::capture(),
        }
    }
}

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ctx) = self.contexts.last() {
            write!(f, "{}", ctx.message)
        } else if let Some(src) = &self.source {
            write!(f, "{}", src)
        } else {
            write!(f, "unknown error")
        }
    }
}

impl fmt::Debug for AnyError {
    /*
    example of how this looks without LUHTWIN_FULL or RUST_BACKTRACE
    Error:
    LUHTWIN_FULL=1 to see full errors <3
      1: failed to load configuration
      2: application startup failed
    source: config.json not found
    backtrace:
    disabled backtrace

    example with LUHTWIN_FULL no RUST_BACKTRACE
    1: failed to load configuration
        - file: "src/lib.rs"
        - line: 466

    2: application startup failed
        - file: "src/tests.rs"
        - line: 329
        - doc link: "https://docs.example.com/startup-errors"
        - issues: ["#123", "#456"]
        - metadata: {"version": "1.0.0", "environment": "production"}

    source: config.json not found
    backtrace:
    disabled backtrace

    with backtrace too

    1: failed to load configuration
        - file: "src/lib.rs"
        - line: 494
  
    2: application startup failed
        - file: "src/tests.rs"
        - line: 329
        - doc link: "https://docs.example.com/startup-errors"
        - issues: ["#123", "#456"]
        - metadata: {"environment": "production", "version": "1.0.0"}
  
    source: config.json not found
    backtrace:
         0: std::backtrace_rs::backtrace::libunwind::trace
                   at /rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/std/src/../../backtrace/src/backtrace/libunwind.rs:116:5
         1: std::backtrace_rs::backtrace::trace_unsynchronized
                   at /rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/std/src/../../backtrace/src/backtrace/mod.rs:66:5
         2: std::backtrace::Backtrace::create
                   at /rustc/4d91de4e48198da2e33413efdcd9cd2cc0c46688/library/std/src/backtrace.rs:331:13
         3: luhtwin::AnyError::new
                   at ./src/lib.rs:294:24
         4: <core::result::Result<T,E> as luhtwin::Context<T>>::context::{{closure}}
                   at ./src/lib.rs:494:26
         5: core::result::Result<T,E>::map_err
                   at /Users/crack/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/core/src/result.rs:856:27
         6: <core::result::Result<T,E> as luhtwin::Context<T>>::context
                   at ./src/lib.rs:494:9
         7: luhtwin::tests::print_error_formats_demonstration
               at ./src/tests.rs:325:29
    etc....

    */

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut print_full = false;

        match env::var("LUHTWIN_FULL") {
            Ok(set) => {
                if set == "1" {
                    print_full = true;
                }
            }
            Err(_) => {
                writeln!(f, "LUHTWIN_FULL=1 to see full errors <3")?;
            }
        }

        if !self.contexts.is_empty() {
            for (i, ctx) in self.contexts.iter().enumerate() {
                if print_full {
                    writeln!(f, "  {}: {}", i + 1, ctx)?;
                } else {
                    writeln!(f, "  {}: {}", i + 1, ctx.message)?;
                }
            }
        } else {
            writeln!(f, "no contexts")?;
        }

        if let Some(src) = &self.source {
            writeln!(f, "source: {}", src)?;
        }

        writeln!(f, "backtrace:\n{}", self.backtrace)?;
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

/// # LuhTwin 
///
/// A wrapper type for a Result with an AnyError it is just an alias though
/// gives you access through traits such as `luhtwin::Twin`, `luhtwin::Wrap`,
/// `luhtwin::Encase` and `luhtwin::Context` to chain different errors together
/// of different types
///
/// ## Examples
///
/// Encasing pre-existing LuhTwin Errors
///
/// ```ignore
/// fn this_fails() -> LuhTwin<()> {
///     Err(io::Error::new(io::ErrorKind::Other, "file missing")
///             .twin())
/// }
/// fn main() -> LuhTwin<()> {
///    this_fails().encase(|| "it failed like expected")?;
///    Ok(())
/// }
/// ```
///
/// Wrapping pre-existing other errors
///
/// ```ignore
/// fn this_fails() -> Result<(), io::Error> {
///     Err(io::Error::new(io::ErrorKind::Other, "idk what happened"))
/// }
/// fn main() -> LuhTwin<()> {
///     this_fails().wrap(|| "it failed")?;
///     Ok(())
/// }
/// ```
///
/// Context Chaining
/// 
/// ```ignore
/// let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "root"));
/// let result = err
///     .context("loading data")
///     .context("initializing system");
/// assert!(result.is_err());
/// let e = result.unwrap_err();
/// assert_eq!(e.to_string(), "initializing system");
/// let src1 = e.source().unwrap();
/// assert_eq!(src1.to_string(), "loading data");
/// let src2 = src1.source().unwrap();
/// assert_eq!(src2.to_string(), "root");
/// ```
pub type LuhTwin<T> = Result<T, AnyError>;

impl From<&str> for AnyError {
    fn from(str: &str) -> Self {
        AnyError::new(at!(str).into_error())
    }
}

impl From<String> for AnyError {
    fn from(str: String) -> Self {
        AnyError::new(at!(str).into_error())
    }
}

impl From<std::io::Error> for AnyError {
    fn from(err: std::io::Error) -> Self {
        AnyError::new(err)
    }
}

impl From<std::fmt::Error> for AnyError {
    fn from(err: std::fmt::Error) -> Self {
        AnyError::new(err)
    }
}

impl From<std::string::FromUtf8Error> for AnyError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AnyError::new(err)
    }
}

/// # BigTwin
///
/// A generic alias for a Result
pub type BigTwin<T, E> = Result<T, E>;

/// # Twin
///
/// Converts any `Result<T, E>` where `E: Error + Send + Sync + 'static` into a `LuhTwin<T>`.
///
/// This trait is used when you have errors that don't have `From` implementations defined
/// for `AnyError`, allowing you to manually convert them.
///
/// ## Methods
///
/// ## `fn twin(self) -> LuhTwin<T>`
///
/// Converts a `Result<T, E>` into a `Result<T, AnyError>` by wrapping the error.
/// This is the simplest way to convert foreign error types into your error handling system.
///
/// ## `fn twin_with<F>(self, f: F) -> LuhTwin<T> where F: FnOnce(AnyError) -> AnyError`
///
/// Converts a `Result<T, E>` into a `Result<T, AnyError>` and then passes the `AnyError`
/// through a transformation function. Useful for adding context during the conversion.
///
/// ## Examples
///
/// ```ignore
/// use std::fs::File;
///
/// // Simple conversion
/// fn open_file() -> LuhTwin<File> {
///     File::open("config.json").twin()
/// }
///
/// // With transformation
/// fn open_with_context() -> LuhTwin<File> {
///     File::open("config.json")
///         .twin_with(|e| e.with_context(at!("failed to open config")))
/// }
/// ```
pub trait Twin<T> {
    fn twin(self) -> LuhTwin<T>;
    fn twin_with<F>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce(AnyError) -> AnyError;
}

impl<T, E> Twin<T> for BigTwin<T, E>
where
    E: Error + Send + Sync + 'static
{
    /// Converts a `Result<T, E>` into a `Result<T, AnyError>` by wrapping the error.
    /// This is the simplest way to convert foreign error types into your error handling system.
    fn twin(self) -> LuhTwin<T> {
        self.map_err(|e| AnyError::new(e))
    }

    /// Converts a `Result<T, E>` into a `Result<T, AnyError>` and then passes the `AnyError`
    /// through a transformation function. Useful for adding context during the conversion.
    fn twin_with<F>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce(AnyError) -> AnyError,
    {
        self.map_err(|e| {
            let err = AnyError::new(e);
            f(err)
        })
    }
}

/// # Context
///
/// Provides context chaining for any `Result<T, E>` where `E: Error + Send + Sync + 'static`
///
/// ## Methods
///
/// ## `fn context<C>(self, msg: C) -> LuhTwin<T> where C: fmt::Display`
///
/// Converts the error into an `AnyError` and adds a context message. The message is
/// evaluated immediately.
///
/// ## `fn with_context<C, F>(self, f: F) -> LuhTwin<T> where C: fmt::Display, F: FnOnce() -> C`
///
/// Lazily converts the error into an `AnyError` and adds a context message. The closure
/// is only called if there's an error, making it more efficient when success is common.
///
/// ## Examples
///
/// ```ignore
/// let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "root"));
/// 
/// // Immediate evaluation
/// let result = err.context("loading data");
///
/// // Lazy evaluation (closure only runs on error)
/// let result = err.with_context(|| format!("loading file: {}", path));
/// ```
pub trait Context<T> {
    fn context<C>(self, msg: C) -> LuhTwin<T> 
    where
        C: fmt::Display;

    fn with_context<C, F>(self, f: F) -> LuhTwin<T>
    where
        C: fmt::Display,
        F: FnOnce() -> C;
}

impl<T, E> Context<T> for BigTwin<T, E>
where
    E: Error + Send + Sync + 'static,
{
    /// Converts the error into an `AnyError` and adds a context message. The message is
    /// evaluated immediately.
    fn context<C>(self, msg: C) -> LuhTwin<T>
    where
        C: fmt::Display,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!("{}", msg)))
    }

    /// Lazily converts the error into an `AnyError` and adds a context message. The closure
    /// is only called if there's an error, making it more efficient when success is common.
    fn with_context<C, F>(self, f: F) -> LuhTwin<T>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!("{}", f())))
    }
}

/// # Wrap
///
/// Wraps any `Result<T, E>` error with additional context, combining the context message
/// with the original error message. Don't use this on a pre-existing `luhtwin::LuhTwin`
/// because it will lead to malformed error messages please see `luhtwin::Encase` trait
///
/// ## Methods
///
/// ## `fn wrap<F, C>(self, f: F) -> LuhTwin<T> where F: FnOnce() -> C, C: fmt::Display`
///
/// Converts the error to an `AnyError` and adds context in the format: `"context: original_error"`.
/// The closure is only evaluated if there's an error.
///
/// ## Examples
///
/// ```ignore
/// fn read_config() -> Result<String, io::Error> {
///     std::fs::read_to_string("config.json")
/// }
///
/// fn main() -> LuhTwin<()> {
///     let config = read_config().wrap(|| "failed to load configuration")?;
///     // Error will display as: "failed to load configuration: No such file or directory"
///     Ok(())
/// }
/// ```
pub trait Wrap<T> {
    fn wrap<F, C>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce() -> C,
        C: fmt::Display;
}

impl<T, E> Wrap<T> for BigTwin<T, E>
where
    E: Error + Send + Sync + 'static,
{
    /// Converts the error to an `AnyError` and adds context in the format: `"context: original_error"`.
    /// The closure is only evaluated if there's an error.
    fn wrap<F, C>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce() -> C,
        C: fmt::Display
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", f(), e);
            AnyError::new(e).with_context(at!(msg))
        })
    }
}

/// # Encase
///
/// Adds an additional layer of context to an existing `LuhTwin<T>` result, creating
/// a chain of contextual information.
///
/// ## Methods
///
/// ## `fn encase<F, C>(self, f: F) -> LuhTwin<T> where F: FnOnce() -> C, C: fmt::Display`
///
/// Adds a new context layer to an existing `AnyError`. This is useful for adding
/// context as errors bubble up through different layers of your application.
///
/// ## Examples
///
/// ```ignore
/// fn load_user(id: u32) -> LuhTwin<User> {
///     database_query(id)
///         .wrap(|| format!("failed to query user {}", id))?
/// }
///
/// fn main() -> LuhTwin<()> {
///     load_user(123)
///         .encase(|| "initialization failed")?;
///     // Error chain: initialization failed -> failed to query user 123: connection timeout
///     Ok(())
/// }
/// ```
pub trait Encase<T> {
    fn encase<F, C>(self, f: F) -> LuhTwin<T>
    where 
        F: FnOnce() -> C,
        C: fmt::Display;
}

impl<T> Encase<T> for LuhTwin<T> {
    /// Adds a new context layer to an existing `AnyError`. This is useful for adding
    /// context as errors bubble up through different layers of your application.
    fn encase<F, C>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce() -> C,
        C: fmt::Display
    {
        self.map_err(|e| {
            e.with_context(at!("{}", f()))
        })
    }
}

/// Creates an `ErrorContext` with automatic file and line information.
///
/// This macro is the primary way to create error contexts with location tracking.
/// It automatically captures `file!()` and `line!()` information.
///
/// ## Syntax
///
/// - `at!()` - Creates context with message "unknown error"
/// - `at!("message")` - Creates context with the given message
/// - `at!("format {}", arg)` - Creates context with formatted message
///
/// ## Examples
///
/// ```ignore
/// // Simple message
/// return Err(at!("configuration invalid").into());
///
/// // Formatted message
/// return Err(at!("user {} not found", user_id).into());
///
/// // Chain with attachments
/// return Err(at!("database error")
///     .attach("query", sql)
///     .attach("params", params)
///     .into());
/// ```
#[macro_export]
macro_rules! at {
    () => {
        $crate::ErrorContext::new("unknown error")
            .file(file!())
            .line(line!())
    };
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::ErrorContext::new(format!($fmt, $($arg),+))
            .file(file!())
            .line(line!())
    };
    ($msg:expr) => {
        $crate::ErrorContext::new($msg)
            .file(file!())
            .line(line!())
    };
}

/// Early return with an error, similar to `anyhow::bail!`.
///
/// This macro creates an `io::Error` with the formatted message and immediately
/// returns it wrapped in an `AnyError`.
///
/// ## Examples
///
/// ```ignore
/// fn validate_age(age: i32) -> LuhTwin<()> {
///     if age < 0 {
///         bail!("age cannot be negative: {}", age);
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::AnyError::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!($($arg)*),
        )))
    };
}

/// Assert a condition and bail with an error if it's false.
///
/// Similar to `anyhow::ensure!`, this combines a condition check with `bail!`.
/// If the condition is false, it returns early with the formatted error message.
///
/// ## Examples
///
/// ```ignore
/// fn process_data(data: &[u8]) -> LuhTwin<()> {
///     ensure!(!data.is_empty(), "data cannot be empty");
///     ensure!(data.len() < 1024, "data too large: {} bytes", data.len());
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::bail!($($arg)*);
        }
    };
}
