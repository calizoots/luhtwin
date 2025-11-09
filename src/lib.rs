//! # LuhTwin Error Handling
//!
//! Dont get comfortable with this library... in dev stages
//! Prone to changes to api
//! A horrible error-handling framework in Rust.
//! Supports context chains, severity levels, strucutred metadata
//! backtraces, and pretty printing. Designed to complicate your
//! Error experience in rust
//!
//! <br>
//!
//! This library provides [`luhtwin::LuhTwin`][LuhTwin] and
//! [`luhtwin::AnyError`][AnyError], horrible ways to handle
//! your errors. for usage look below.
//!
//! <br>
//!
//! ## Examples
//!
//! these are just basic for now will add more this is still in dev stages
//!
//! ```rust
//! use luhtwin::{ensure, LuhTwin};
//! 
//! fn other_test(a: u32, b: u32) -> u32 {
//!     return a + b
//! }
//! 
//! fn main() -> LuhTwin<()> {
//!     let x = other_test(9, 10);
//! 
//!     ensure!(x == 10, "critical error");
//! 
//!     Ok(())
//! }
//! ```
//! 
//! ```rust
//! use luhtwin::{bail, LuhTwin};
//! 
//! fn main() -> LuhTwin<()> {
//!     bail!("bailing immediately");
//! }
//! ```
//! 
//! ```rust
//! use luhtwin::{anyerror, at, LuhTwin, Severity};
//! 
//! fn main() -> LuhTwin<()> {
//!     println!("Hello, world!");
//! 
//!     let err = anyerror!("critical bine")
//!         .doc_link("http://bine.com/docs/criticalbine")
//!         .issues(vec!("#103", "#104"))
//!         .severity(Severity::Critical)
//!         .build();
//! 
//!     let first = at!();
//!     println!("{}", first);
//! 
//!     Err(err)
//! }
//! ```
//!

use std::error::Error;
use std::backtrace::Backtrace;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// Just a helper type for an ErrorSource to keep things consistent
pub type ErrorSource = Box<dyn Error + Send + Sync + 'static>;

/// Severity levels for errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// it indicates the importance of a error or log message
/// pretty self explanatory... 
pub enum Severity {
    // im not adding individual messages i think people are smarter
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // in future love to make these customisable <3
            Severity::Debug => write!(f, "DEBUG"),
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            // 5 figure duppy
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}

/// The strucutre for all metadata for a given AnyError
///
/// does look a bit clumsy but dw - s.c 2025 |c|
/// 
/// To create a `ErrorContext` on the fly you can use
/// `at!(...)` example...
///
/// ```
/// fn main() -> LuhTwin<()> {
///     let err = anyerror!("test error")
///         .severity(Severity::Critical)
///         .build()
///         .with_context(at!("chained error"));
///     return Err(err)
/// }
/// ```
/// or you can see here if you intention to is to make an `AnyError`
/// you can use `anyerror!(...)` macro
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub msg: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub doc_link: Option<String>,
    pub issues: Vec<String>,
    pub metadata: Vec<(String, String)>,
    pub severity: Severity,
}

impl fmt::Display for ErrorContext {
    /// will expand to something like
    ///
    /// error: critical bine error
    /// --> src/main.rs:12
    ///  documentation: https://bine.com/docs/critical-bine-error/
    ///  related issues:
    ///      - #104
    ///      - #512
    ///      - #3042
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "error: {}", self.msg)?;

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
            writeln!(f, " related issues:")?;
            for issue in &self.issues {
                writeln!(f, "    - {}", issue)?;
            }
        }

        Ok(())
    }
}

/// Just some basic combinators for ease
impl ErrorContext {
    pub fn with_doc_link(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }

    pub fn with_issues<I, S>(mut self, issues: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.issues.extend(issues.into_iter().map(Into::into));
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: fmt::Display,
    {
        self.metadata.push((key.into(), value.to_string()));
        self
    }
}

/// `luhtwin::at!(...)` produces an ErrorContext easily filling out basic metadata
/// such as file and line and has a few different options
///
/// you can also use different combinators such as .with_severity(),
/// .with_metadata() or .with_issues() etc... to produce an error
/// with your specified verboseness
///
/// ## Example
///
/// ```
/// fn main() -> LuhTwin<()> {
///     let first = at!("first bine");
///     // error: first bine
///     //  --> src/main.rs:4
///     let second = at!("second bine", Severity::Critical);
///     // no output difference just trust me bro else sym
///     let third = at!();
///     // error: unknown error
///     //  --> src/main.rs:9
/// }
/// ```
#[macro_export]
macro_rules! at {
    ($msg:expr) => {
        $crate::ErrorContext {
            msg: $msg.to_string(),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $crate::Severity::Error,
            metadata: vec![],
        }
    };
    ($msg:expr, $severity:expr) => {
        $crate::ErrorContext {
            msg: $msg.to_string(),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $severity,
            metadata: vec![],
        }
    };
    () => {
        $crate::ErrorContext {
            msg: "unknown error".to_string(),
            file: Some(file!().to_string()),
            line: Some(line!()),
            doc_link: None,
            issues: vec![],
            severity: $crate::Severity::Error,
            metadata: vec![],
        }
    };
}

/// `luhtwin::AnyError` is an horrible way to manage your errors
/// it has a lists of contexts a source and a backtrace with item
/// look at `luhtwin::ErrorContext` for more information on what
/// type of metadata you can save and display in contexts
///
/// # Examples
///
/// ```rust
/// use luhtwin::{AnyError, ErrorContext};
///
/// fn may_fail(value: i32) -> Result<(), AnyError> {
///     if value % 2 == 0 {
///         Ok(())
///     } else {
///         let mut err = AnyError::new("odd value error");
///         err.add_context(ErrorContext::new("failed in may_fail"));
///         Err(err)
///     }
/// }
///
/// fn main() {
///     match may_fail(3) {
///         Ok(_) => println!("success"),
///         Err(e) => {
///             let mut e = e;
///             e.add_context(ErrorContext::new("while running main"));
///             e.display_full(); // verbose output with contexts and backtrace
///         }
///     }
/// }
/// ```
pub struct AnyError {
    contexts: Vec<ErrorContext>,
    source: Option<ErrorSource>,
    backtrace: Backtrace,
    logged: std::sync::atomic::AtomicBool,
}

pub type LuhTwin<T> = Result<T, AnyError>;
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

    pub fn mark_logged(&self) {
        self.logged.store(true, std::sync::atomic::Ordering::SeqCst);
    }

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
    fn clone_box(&self) -> Box<dyn CloneableError>;
    fn as_error(&self) -> &(dyn Error + 'static);
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
    fn make_clonable(&self) -> Box<dyn CloneableError> {
        // If it's already a CloneableError, just clone it
        if let Some(clonable) = (self as &dyn std::any::Any).downcast_ref::<Box<dyn CloneableError>>() {
            return clonable.clone_box();
        }
        // Otherwise, wrap it in NonCloneableWrapper
        Box::new(NonCloneableWrapper::new(self))
    }
}

impl Clone for NonCloneableWrapper {
    fn clone(&self) -> Self {
        Self { msg: self.msg.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct AnyErrorBuilder {
    ctx: ErrorContext,
    source: Option<Arc<ErrorSourceWrapper>>,
}

impl AnyErrorBuilder {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            ctx: ErrorContext {
                msg: msg.into(),
                file: Some(file!().to_string()),
                line: Some(line!()),
                doc_link: None,
                issues: vec![],
                metadata: vec![],
                severity: Severity::Error,
            },
            source: None,
        }
    }

    pub fn doc_link(mut self, link: impl Into<String>) -> Self {
        self.ctx.doc_link = Some(link.into());
        self
    }

    pub fn issues<I, S>(mut self, issues: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.ctx.issues.extend(issues.into_iter().map(Into::into));
        self
    }

    pub fn metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: fmt::Display,
    {
        self.ctx.metadata.push((key.into(), value.to_string()));
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.ctx.severity = severity;
        self
    }

    pub fn source<E>(mut self, err: E) -> Self
    where
        E:  Error + Send + Sync + 'static,
    {
        self.source = Some(Arc::new(ErrorSourceWrapper(err.make_clonable())));
        self
    }

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

#[macro_export]
macro_rules! anyerror {
    ($msg:expr) => {
        $crate::AnyErrorBuilder::new($msg)
    };
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

impl AnyError {
    pub fn with_context(mut self, ctx: ErrorContext) -> Self {
        self.contexts.push(ctx);
        self
    }

    pub fn max_severity(&self) -> Severity {
        self.contexts
            .iter()
            .map(|c| c.severity)
            .max()
            .unwrap_or(Severity::Error)
    }

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
                Severity::Critical | Severity::Error => RED,
                Severity::Warning => YELLOW,
                Severity::Info => BLUE,
                Severity::Debug => GRAY,
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
            // Show from outermost to innermost (reverse order)
            for (i, ctx) in self.contexts.iter().rev().enumerate() {
                result.push_str(&format!("  {}. {}\n", i + 1, ctx.msg));
            }
        }
        
        if let Some(src) = &self.source {
            result.push_str(&format!("\n{}caused by:{} {}\n", BOLD, RESET, src));
        }
        
        result
    }

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

    pub fn display_backtrace(&self) -> String {
        format!("{}", self.backtrace)
    }

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
    pub fn root_cause(&self) -> &(dyn Error + 'static) {
        let mut source: &(dyn Error + 'static) = self;
        while let Some(s) = source.source() {
            source = s;
        }
        source
    }

    pub fn iter_sources(&self) -> impl Iterator<Item = &(dyn Error + 'static)> + '_ {
        std::iter::successors(Some(self as &(dyn Error + 'static)), |&e| e.source())
    }
}

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
    fn context<C>(self, msg: C) -> Result<T, AnyError>
    where
        C: fmt::Display,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!(msg)))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, AnyError>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!(f())))
    }
}

pub trait MapErrExt<T> {
    fn map_err_context<F, C>(self, f: F) -> LuhTwin<T>
    where
        F: FnOnce() -> C,
        C: fmt::Display;
}

impl<T, E> MapErrExt<T> for BigTwin<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn map_err_context<F, C>(self, f: F) -> LuhTwin<T>
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

pub trait LogError<T> {
    fn log_error(self) -> Self;
    
    fn log_error_with(self, prefix: &str) -> Self;
    
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

#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::AnyError::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!($($arg)*),
        )))
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::bail!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! context {
    ($res:expr, $msg:expr) => {
        $res.with_context(|| $msg)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::error::Error as StdError;

    fn root_cause<'a>(err: &'a (dyn StdError + 'static)) -> &'a (dyn StdError + 'static) {
        let mut current = err;
        while let Some(source) = current.source() {
            current = source;
        }
        current
    }

    #[test]
    fn context_adds_message_and_preserves_source() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "file missing"));
        let result = err.context("reading config");

        assert!(result.is_err());
        let e = result.unwrap_err();

        assert_eq!(e.to_string(), "reading config");

        assert_eq!(e.source().unwrap().to_string(), "file missing");
    }

    #[test]
    fn chained_context_produces_nested_sources() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "root"));
        let result = err
            .context("loading data")
            .context("initializing system");

        assert!(result.is_err());
        let e = result.unwrap_err();

        assert_eq!(e.to_string(), "initializing system");

        let src1 = e.source().unwrap();
        assert_eq!(src1.to_string(), "loading data");

        let src2 = src1.source().unwrap();
        assert_eq!(src2.to_string(), "root");
    }

    #[test]
    fn context_chain_root_cause_is_original_error() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "disk full"));
        let result = err
            .context("saving file")
            .context("processing upload")
            .context("user request");

        let e = result.unwrap_err();
        let cause = root_cause(&e);
        assert_eq!(cause.to_string(), "disk full");
    }

    #[test]
    fn map_err_produces_combined_message() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "low memory"));
        let result = err.map_err_context(|| "while running benchmark");
        // let result = err.map_err(|e| format!("while running benchmark: {}", e));

        assert!(result.is_err());
        let e = result.unwrap_err();
        let msg = e.to_string();
        
        assert!(msg.contains("while running benchmark"));
        assert!(msg.contains("low memory"));
    }

    #[test]
    fn bail_macro_returns_early_with_message() {
        fn fail() -> LuhTwin<()> {
            bail!("fatal error occurred");
        }

        let result = fail();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "fatal error occurred");
    }

    #[test]
    fn bail_macro_with_format_args() {
        fn fail_with_data(x: i32) -> LuhTwin<()> {
            bail!("invalid number: {}", x);
        }

        let result = fail_with_data(99);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "invalid number: 99");
    }

    #[test]
    fn ensure_macro_allows_valid_condition() {
        fn check_positive(x: i32) -> LuhTwin<()> {
            ensure!(x > 0, "expected positive, got {}", x);
            Ok(())
        }

        let ok = check_positive(10);
        assert!(ok.is_ok());
    }

    #[test]
    fn ensure_macro_bails_on_failure() {
        fn check_positive(x: i32) -> LuhTwin<()> {
            ensure!(x > 0, "expected positive, got {}", x);
            Ok(())
        }

        let err = check_positive(-5);
        
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().to_string(), "expected positive, got -5");
    }

    #[test]
    fn context_and_macros_mix_well() {
        fn process_file() -> LuhTwin<()> {
            let file: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "IO fail"));
            file.context("opening file")?;
            Ok(())
        }

        let result = process_file();
        assert!(result.is_err());
        let e = result.unwrap_err();
        assert_eq!(e.to_string(), "opening file");
        assert_eq!(e.source().unwrap().to_string(), "IO fail");
    }

    #[test]
    fn ensure_and_bail_can_coexist() {
        fn maybe_fail(x: i32) -> LuhTwin<()> {
            ensure!(x != 0, "zero not allowed");
            if x == 42 {
                bail!("meaning of life error");
            }
            Ok(())
        }

        let zero = maybe_fail(0);
        assert!(zero.is_err());
        assert_eq!(zero.unwrap_err().to_string(), "zero not allowed");

        let meaning = maybe_fail(42);
        assert!(meaning.is_err());
        assert_eq!(meaning.unwrap_err().to_string(), "meaning of life error");

        let ok = maybe_fail(7);
        assert!(ok.is_ok());
    }

    use std::thread;
    use std::sync::Arc;

    #[test]
    fn anyerror_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AnyError>();
    }

    #[test]
    fn anyerror_can_be_sent_across_threads() {
        let err = AnyError::new(std::io::Error::new(std::io::ErrorKind::Other, "thread error"))
            .with_context(at!("from worker thread"));

        let shared = Arc::new(err);
        let thread_err = shared.clone();

        let handle = thread::spawn(move || {
            assert_eq!(thread_err.to_string(), "from worker thread");
            assert!(thread_err.source().unwrap().to_string().contains("thread error"));
        });

        handle.join().expect("thread should finish");
    }

    #[test]
    fn anyerror_without_message_defaults_to_unknown() {
        let e = AnyError { contexts: vec!(), source: None, backtrace: Backtrace::capture(), logged: false.into() };
        assert_eq!(e.to_string(), "unknown error");
        assert!(e.source().is_none());
    }

    #[test]
    fn anyerror_with_source_but_no_message_displays_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "inner cause");
        let e = AnyError::new(inner);
        assert_eq!(e.to_string(), "inner cause");
    }

    #[derive(Debug)]
    struct CustomErr;
    impl fmt::Display for CustomErr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "custom error occurred")
        }
    }
    impl Error for CustomErr {}

    #[test]
    fn context_works_with_custom_non_io_error() {
        let err: Result<(), CustomErr> = Err(CustomErr);
        let result = err.context("running plugin");
        let e = result.unwrap_err();
        assert_eq!(e.to_string(), "running plugin");
        assert_eq!(e.source().unwrap().to_string(), "custom error occurred");
    }

    #[test]
    fn display_shows_top_message_only() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "root failure"));
        let e = err.context("loading config").context("starting system").unwrap_err();
        assert_eq!(e.to_string(), "starting system");
    }

    #[test]
    fn root_cause_finds_deepest_source() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "deep root"));
        let result = err.context("layer 1").context("layer 2").context("layer 3");
        let e = result.unwrap_err();
        let cause = root_cause(&e);
        assert_eq!(cause.to_string(), "deep root");
    }

    #[test]
    fn map_err_context_handles_empty_message() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "missing data"));
        let result = err.map_err_context(|| "");
        let e = result.unwrap_err();
        assert!(e.to_string().contains("missing data"));
    }

    #[test]
    fn with_context_closure_is_lazy() {
        let mut called = false;
        let err: Result<(), io::Error> = Ok(());
        let _ = err.with_context(|| {
            called = true;
            "this should not be called"
        });
        assert!(!called, "closure should not be called on Ok");
    }
    
    #[test]
    fn with_context_closure_is_called_on_err() {
        let mut called = false;
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "fail"));
        let _ = err.with_context(|| {
            called = true;
            "context added"
        });
        assert!(called, "closure should be called on Err");
    }
    
    #[test]
    fn max_severity_returns_highest() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "test"))
            .with_context(at!("debug msg", Severity::Debug))
            .with_context(at!("critical msg", Severity::Critical))
            .with_context(at!("info msg", Severity::Info));
        
        assert_eq!(err.max_severity(), Severity::Critical);
    }
    
    #[test]
    fn max_severity_defaults_to_error() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "test"));
        assert_eq!(err.max_severity(), Severity::Error);
    }
    
    #[test]
    fn error_context_builder_methods() {
        let ctx = at!("test error")
            .with_doc_link("https://docs.example.com/error")
            .with_issues(vec!["#123", "#456"])
            .with_severity(Severity::Warning)
            .with_metadata("user_id", 42)
            .with_metadata("request_id", "abc-123");
        
        assert_eq!(ctx.doc_link, Some("https://docs.example.com/error".to_string()));
        assert_eq!(ctx.issues.len(), 2);
        assert_eq!(ctx.severity, Severity::Warning);
        assert_eq!(ctx.metadata.len(), 2);
    }
    
    #[test]
    fn to_log_format_produces_structured_output() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "source error"))
            .with_context(at!("test message"));
        
        let log = err.to_log_format();
        assert!(log.contains("message=\"test message\""));
        assert!(log.contains("severity=ERROR"));
        assert!(log.contains("source=\"source error\""));
    }
    
    #[test]
    fn to_log_format_escapes_quotes() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "error with \"quotes\""))
            .with_context(at!("message with \"quotes\""));
        
        let log = err.to_log_format();
        assert!(log.contains("\\\""));
    }
    
    #[test]
    fn iter_sources_traverses_entire_chain() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "root"));
        let e = err.context("layer1").context("layer2").unwrap_err();
        
        let sources: Vec<String> = e.iter_sources().map(|s| s.to_string()).collect();
        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0], "layer2");
        assert_eq!(sources[1], "layer1");
        assert_eq!(sources[2], "root");
    }
    
    #[test]
    fn root_cause_on_anyerror_directly() {
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::Other, "original"));
        let e = err.context("wrapper").unwrap_err();
        
        let root = e.root_cause();
        assert_eq!(root.to_string(), "original");
    }
    
    #[test]
    fn display_contexts_shows_all_contexts() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "source"))
            .with_context(at!("first context"))
            .with_context(at!("second context"));
        
        let display = err.display_contexts();
        assert!(display.contains("1: second context"));
        assert!(display.contains("-> 2: first context"));
    }
    
    #[test]
    fn display_contexts_includes_doc_links_and_issues() {
        let ctx = at!("error with metadata")
            .with_doc_link("https://example.com")
            .with_issues(vec!["issue-1", "issue-2"]);
        
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "test"))
            .with_context(ctx);
        
        let display = err.display_contexts();
        assert!(display.contains("[doc: https://example.com]"));
        assert!(display.contains("[issues: issue-1, issue-2]"));
    }
    
    #[test]
    fn display_contexts_tree_formats_correctly() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "source"))
            .with_context(at!("first"))
            .with_context(at!("second"));
        
        let tree = err.display_contexts_tree();
        println!("{}", tree);
        assert!(tree.contains("├─ second"));

        assert!(tree.contains("└─ first"));
    }
    
    #[test]
    fn log_error_trait_does_not_consume_result() {
        let err: LuhTwin<()> = Err(AnyError::new(io::Error::new(io::ErrorKind::Other, "test")));
        let result = err.log_error();
        assert!(result.is_err());
    }
    
    #[test]
    fn log_once_marks_error_as_logged() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "test"));
        assert!(!err.is_logged());
        
        let result: LuhTwin<()> = Err(err);
        let _ = result.log_once();
    }
    
    #[test]
    fn multiple_contexts_preserve_order() {
        let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "root"))
            .with_context(at!("first"))
            .with_context(at!("second"))
            .with_context(at!("third"));
        
        assert_eq!(err.contexts.len(), 3);
        assert_eq!(err.contexts[0].msg, "first");
        assert_eq!(err.contexts[1].msg, "second");
        assert_eq!(err.contexts[2].msg, "third");
    }
    
    #[test]
    fn from_implementations_work() {
        let io_err = io::Error::new(io::ErrorKind::Other, "io");
        let any_err: AnyError = io_err.into();
        assert!(any_err.source().is_some());
        
        let fmt_err = std::fmt::Error;
        let any_err: AnyError = fmt_err.into();
        assert!(any_err.source().is_some());
    }
    
    #[test]
    fn at_macro_with_no_args() {
        let ctx = at!();
        assert_eq!(ctx.msg, "unknown error");
        assert!(ctx.file.is_some());
        assert!(ctx.line.is_some());
    }
    
    #[test]
    fn ensure_macro_with_complex_conditions() {
        fn validate(x: i32, y: i32) -> LuhTwin<()> {
            ensure!(x > 0 && y > 0, "both values must be positive: x={}, y={}", x, y);
            ensure!(x < y, "x must be less than y: {} >= {}", x, y);
            Ok(())
        }
        
        assert!(validate(5, 10).is_ok());
        assert!(validate(-1, 10).is_err());
        assert!(validate(10, 5).is_err());
    }
    
    #[test]
    fn print_error_formats_demonstration() {
        println!("\n=== ERROR FORMAT DEMONSTRATION ===\n");
        
        let err: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::NotFound, "config.json not found"));
        let err_with_metadata = err
            .context("failed to load configuration")
            .unwrap_err()
            .with_context(
                at!("application startup failed", Severity::Critical)
                    .with_doc_link("https://docs.example.com/startup-errors")
                    .with_issues(vec!["#123", "#456"])
                    .with_metadata("version", "1.0.0")
                    .with_metadata("environment", "production")
            );
        
        println!("1. DISPLAY (to_string):");
        println!("{}\n", err_with_metadata);
        
        println!("2. DISPLAY_PRETTY:");
        println!("{}\n", err_with_metadata.display_pretty());
        
        println!("3. DISPLAY_CONTEXTS:");
        println!("{}\n", err_with_metadata.display_contexts());
        
        println!("4. DISPLAY_CONTEXTS_TREE:");
        println!("{}\n", err_with_metadata.display_contexts_tree());
        
        println!("5. TO_LOG_FORMAT:");
        println!("{}\n", err_with_metadata.to_log_format());
        
        println!("6. DISPLAY_FULL:");
        println!("{}\n", err_with_metadata.display_full());
        
        println!("=== END DEMONSTRATION ===\n");
    }
    
    #[test]
    fn severity_display_formats() {
        assert_eq!(Severity::Debug.to_string(), "DEBUG");
        assert_eq!(Severity::Info.to_string(), "INFO");
        assert_eq!(Severity::Warning.to_string(), "WARN");
        assert_eq!(Severity::Error.to_string(), "ERROR");
        assert_eq!(Severity::Critical.to_string(), "CRIT");
    }
    
    #[test]
    fn error_context_display_with_all_fields() {
        let ctx = ErrorContext {
            msg: "test error".to_string(),
            file: Some("main.rs".to_string()),
            line: Some(42),
            doc_link: Some("https://example.com/docs".to_string()),
            issues: vec!["issue-1".to_string(), "issue-2".to_string()],
            metadata: vec![],
            severity: Severity::Error,
        };
        
        let display = format!("{}", ctx);
        assert!(display.contains("test error"));
        assert!(display.contains("main.rs:42"));
        assert!(display.contains("https://example.com/docs"));
        assert!(display.contains("issue-1"));
        assert!(display.contains("issue-2"));
    }
}
