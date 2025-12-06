use super::*;

use std::collections::HashMap;
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
    let result = err.wrap(|| "while running benchmark");

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
    let e = AnyError { contexts: vec!(), source: None, backtrace: Backtrace::capture() };
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
    let result = err.wrap(|| "");
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
fn multiple_contexts_preserve_order() {
    let err = AnyError::new(io::Error::new(io::ErrorKind::Other, "root"))
        .with_context(at!("first"))
        .with_context(at!("second"))
        .with_context(at!("third"));
    
    assert_eq!(err.contexts.len(), 3);
    assert_eq!(err.contexts[0].message, "first");
    assert_eq!(err.contexts[1].message, "second");
    assert_eq!(err.contexts[2].message, "third");
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
    let mut meta = HashMap::new();
    meta.insert("version", "1.0.0");
    meta.insert("environment", "production");
    let err_with_metadata = err
        .context("failed to load configuration")
        .unwrap_err()
        .with_context(
            at!("application startup failed")
                .attach("doc link", "https://docs.example.com/startup-errors")
                .attach("issues", vec!["#123", "#456"])
                .attach("metadata", meta)
        );
    
    println!("{:?}\n", err_with_metadata);
    
    println!("=== END DEMONSTRATION ===\n");
}

fn test_wrap_luhtwin_helper() -> LuhTwin<()> {
    let err: LuhTwin<()> = Err(io::Error::new(io::ErrorKind::NotFound, "config.json not found"))
        .wrap(|| "failed to get config.json");

    err
}

#[test]
fn test_wrap_luhtwin() {
    let err = test_wrap_luhtwin_helper()
        .encase(|| "luhtwin helper error");

    println!("{:?}", err)
}

#[test]
fn error_context_display_with_all_fields() {
    let mut meta = HashMap::new();
    meta.insert("version", "1.0.0");
    meta.insert("environment", "production");
    let ctx = ErrorContext::new("test error")
        .file("main.rs")
        .line(42)
        .attach("doc link", "https://example.com/docs")
        .attach("issues", vec!["issue-1", "issue-2"])
        .attach("metadata", meta);
    
    let display = format!("{}", ctx);
    println!("{}", display);
    assert!(display.contains("test error"));
    assert!(display.contains("main.rs"));
    assert!(display.contains("42"));
    assert!(display.contains("https://example.com/docs"));
    assert!(display.contains("issue-1"));
    assert!(display.contains("issue-2"));
}
