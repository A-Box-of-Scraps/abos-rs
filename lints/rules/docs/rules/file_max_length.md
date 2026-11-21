# file_max_length

Warns when a local Rust source file exceeds the configured line limit. Each `.rs` file is counted separately, including
blank lines and comments. Files under `OUT_DIR` are skipped.

Both examples use a limit of three lines.

## Accepted

A file with exactly three lines is within the limit:

```rust
fn main() {
    println!("hello");
}
```

## Rejected

Adding a comment brings the file to four lines. The warning points to line 4, the first line over the limit:

```rust
// Greet the user.
fn main() {
    println!("hello");
}
```

## Options

`max_lines` is the maximum number of lines allowed per file. It defaults to `600`. To use the limit from the examples,
set it in `dylint.toml`:

```toml
[abos_rust_rules.file_max_length]
max_lines = 3
```
