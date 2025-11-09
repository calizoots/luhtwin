# LuhTwin

A horrible Rust library for managing errors.

Dont get comfortable with this library... in dev stages it will be prone to changes to api
<br>
Supports context chains, severity levels, strucutred metadata, backtraces, and pretty printing. 
<br>
Designed to complicate your Error experience in Rust
<br>
made by **yours truely calizoots** <3

## Example

these are just some basic examples will share more with time still in early stages

```rust
use luhtwin::{ensure, LuhTwin};

fn other_test(a: u32, b: u32) -> u32 {
    return a + b
}

fn main() -> LuhTwin<()> {
    let x = other_test(9, 10);

    ensure!(x == 10, "critical error");

    Ok(())
}
```

```rust
use luhtwin::{bail, LuhTwin};

fn main() -> LuhTwin<()> {
    bail!("bailing immediately");
}
```

```rust
use luhtwin::{anyerror, at, LuhTwin, Severity};

fn main() -> LuhTwin<()> {
    println!("Hello, world!");

    let err = anyerror!("critical bine")
        .doc_link("http://bine.com/docs/criticalbine")
        .issues(vec!("#103", "#104"))
        .severity(Severity::Critical)
        .build();

    let first = at!();
    println!("{}", first);

    Err(err)
}
```
