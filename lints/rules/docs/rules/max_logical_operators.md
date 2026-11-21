# max_logical_operators

Warns once per boolean chain that exceeds the operator limit. Count each `&&` and `||` as one, whether the chain uses
one kind or both. Parentheses, negation, and braces around a bare expression do not reset the count. Named bindings and
function arguments are checked separately. Operators joining `let` chains count too.

This rule checks chain length, not where the chain appears. For example, `if is_paid && is_packed` is within the
default limit but still triggers [`no_inline_logical_conditions`](no_inline_logical_conditions.md). A named binding
has the opposite trade-off: it can keep a condition out of the branch, but its chain must still fit the limit.

## Accepted

With the default limit of three, both decisions fit:

```rust
let ready_to_ship: bool = is_paid && is_packed && has_label && has_address;
let can_dispatch: bool = ready_to_ship && carrier_available;
```

## Rejected

This chain has four operators. Grouping the first two checks does not start a new chain:

```rust
let can_dispatch: bool = (is_paid && is_packed) && has_label && has_address && carrier_available;
```

Name a smaller decision, as above. If a check has side effects, preserve short-circuit evaluation: extracting the right
side of `||` into a predicate call keeps it conditional; computing it in advance does not.

## Options

`max` is the maximum number of operators allowed per chain, inclusive. It defaults to `3`. To warn on even one operator,
set this in `dylint.toml`:

```toml
[abos_rust_rules.max_logical_operators]
max = 0
```
