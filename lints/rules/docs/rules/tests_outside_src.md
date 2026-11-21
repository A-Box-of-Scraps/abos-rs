# tests_outside_src

Warns on registered tests and test-only code under a Cargo package’s `src/`. Move the code outside that directory;
the rule does not require a particular destination.

## Accepted

A declaration in `src/lib.rs` can load an external test module. The tests remain in the same crate and can still access
private items:

```rust
#[cfg(test)]
#[path = "../unit/parser.rs"]
mod parser_tests;
```

A guard such as `#[cfg(any(test, windows))]` is also allowed: the item can exist in a non-test Windows build.

## Rejected

In `src/lib.rs`, both the helper and the test belong outside `src/`:

```rust
#[cfg(all(test, windows))]
fn windows_fixture() {}

#[test]
fn parses_empty_input() {}
```

The rule also checks framework-registered tests, test-only statements, and direct `if cfg!(...)` branches that run only
in tests. A `cfg_attr` that makes an item test-only is treated like a test-only guard; one that merely adds a derive,
such as `#[cfg_attr(test, derive(Debug))]`, is allowed.
