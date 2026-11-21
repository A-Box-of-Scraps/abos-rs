# no_magic_numbers

Warns when an integer or floating-point literal has no direct name. Zero, one, and negative one are exempt.

## Accepted

```rust
const MAX_RETRIES: usize = 3;
if retries < MAX_RETRIES {
    retry();
}
```

A literal assigned directly to a variable or named field also counts as named. For example, `let max_retries = 3;` is
enough; the rule does not require a constant.

## Rejected

```rust
if retries < 3 {
    retry();
}
```

Naming the result of a calculation does not name its operands. For example, `let bytes = kibibytes * 1024;` still warns
on `1024`. Use a name such as `BYTES_PER_KIBIBYTE` for the conversion factor.
