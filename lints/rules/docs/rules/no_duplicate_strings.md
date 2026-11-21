# no_duplicate_strings

Warns once when the same string literal appears at least three times in one file and is at least ten characters long
by default. The rule compares decoded values, so raw and ordinary strings with the same text count together. Uses in
different files do not add up.

## Rejected

```rust
const REQUEST_ID: &str = "x-request-id";
let incoming = request.headers().get("x-request-id");
let outgoing = response.headers().get("x-request-id");
```

The constant initializer counts as the first occurrence. The warning points to the existing constant so you can reuse
it instead of introducing another name.

## Accepted

```rust
const REQUEST_ID: &str = "x-request-id";
let incoming = request.headers().get(REQUEST_ID);
let outgoing = response.headers().get(REQUEST_ID);
```

## Options

Set these in `dylint.toml`. The values shown are the defaults:

```toml
[abos_rust_rules.no_duplicate_strings]
threshold = 3
shortString = 10
ignoreStrings = []
```

- `threshold` is the number of occurrences that triggers a warning. It must be at least `2`.
- `shortString` is the minimum length in Unicode scalar values, not bytes. Set it to `0` to include even empty strings.
- `ignoreStrings` lists exact string values to skip. Entries must be unique.
