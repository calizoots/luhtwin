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
//! ## Core Ideas
//!
//! - **[`AnyError`]** — The main error container that wraps any `Error` type with context chains
//! - **[`ErrorContext`]** — Structured metadata including messages, file/line info, docs, and severity
//! - **[`Wrap`]** — Wrapping existing Results into LuhTwin.
//! - **[`Twin`]** — Transforming existing Results into LuhTwin.
//! - **[`Encase`]** — Encase existing LuhTwins in another layer of context.
//! - **[`LuhTwin<T>`]** — Type alias for `Result<T, AnyError>`, the primary result type
//!
//! docs are ass rn but we finna get to it lmaoo

#[cfg(test)]
mod tests;

use std::backtrace::Backtrace;
use std::env;
use std::error::Error;
use std::any::Any;
use std::fmt::{self, Debug};

pub type ErrorSource = Box<dyn Error + Send + Sync + 'static>;

pub trait ErrorArg: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn display(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

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

pub struct ErrorContext {
    pub message: String,
    pub args: Vec<Box<dyn ErrorArg>>,
}

impl ErrorContext {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            args: vec![]
        }
    }

    pub fn attach_other<E: ErrorArg + 'static>(mut self, arg: E) -> Self {
        self.args.push(Box::new(arg));
        self
    }

    pub fn attach_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        f(&mut self);
        self
    }

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

    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.args.push(Box::new(Arg {
            name: "file".to_string(),
            value: file.into()
        }));
        self
    }

    pub fn line(mut self, line: u32) -> Self {
        self.args.push(Box::new(Arg {
            name: "line".to_string(),
            value: line
        }));
        self
    }

    pub fn into_error(self) -> AnyError {
        self.into()
    }
}

impl From<&str> for ErrorContext {
    fn from(str: &str) -> Self {
       at!(str) 
    }
}

impl From<String> for ErrorContext {
    fn from(str: String) -> Self {
       at!(str) 
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;

        for arg in &self.args {
            writeln!(f, "    - {}: {}", arg.name(), arg.display())?;
        }

        Ok(())
    }
}

pub struct AnyError {
    contexts: Vec<ErrorContext>,
    source: Option<ErrorSource>,
    backtrace: Backtrace,
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
        }
    }

    pub fn with_context(mut self, ctx: ErrorContext) -> Self {
        self.contexts.push(ctx);
        self
    }

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

impl From<ErrorContext> for AnyError {
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
            writeln!(f, "errors:")?;
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

pub type BigTwin<T, E> = Result<T, E>;

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
    fn twin(self) -> LuhTwin<T> {
        self.map_err(|e| AnyError::new(e))
    }

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
    fn context<C>(self, msg: C) -> LuhTwin<T>
    where
        C: fmt::Display,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!("{}", msg)))
    }

    fn with_context<C, F>(self, f: F) -> LuhTwin<T>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| AnyError::new(e).with_context(at!("{}", f())))
    }
}

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

pub trait Encase<T> {
    fn encase<F, C>(self, f: F) -> LuhTwin<T>
    where 
        F: FnOnce() -> C,
        C: fmt::Display;
}

impl<T> Encase<T> for LuhTwin<T> {
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
