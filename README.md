# luhtwin

[<img alt="github" src="https://img.shields.io/badge/github-calizoots/luhtwin-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/calizoots/luhtwin)
[<img alt="crates.io" src="https://img.shields.io/crates/v/luhtwin.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/luhtwin)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-luhtwin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/luhtwin)

> Horrible Error Handling for Rust

`luhtwin` provides a horrible, non-ergonomic error handling system that emphasizes **context accumulation**, **structured diagnostics**, and **flexible formatting**. Built around the `AnyError` type, it allows you to wrap any error with rich metadata and progressively add context as errors bubble up through your application.

## Core Ideas

- **[`AnyError`]** — The main error container that wraps any `Error` type with context chains
- **[`ErrorContext`]** — Structured metadata including messages, file/line info, docs, and severity
- **[`Wrap`]** — Wrapping existing Results into LuhTwin.
- **[`Twin`]** — Transforming existing Results into LuhTwin.
- **[`Encase`]** — Encase existing LuhTwins in another layer of context.
- **[`LuhTwin<T>`]** — Type alias for `Result<T, AnyError>`, the primary result type

> docs are ass rn but we finna get to it lmaoo
> still in development heavily <3333 made with love - s.c
