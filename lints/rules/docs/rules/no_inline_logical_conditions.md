# no_inline_logical_conditions

Warns once per `if`, `else if`, `while`, or match guard that uses an inline `&&` or `||`, even inside a function
argument. Name the decision so the reader can tell what the branch is for. A comparison or a named predicate call does
not need an extra binding.

## Accepted

```rust
let can_ship: bool = is_paid && is_packed;
if can_ship {
    ship();
}
while retries < limit {
    retries += 1;
}
```

## Rejected

```rust
if is_paid && is_packed {
    ship();
}
while queue.has_work() && !cancelled() {
    process_next(&mut queue);
}
```

For the loop, use a named predicate such as `while should_continue(&queue)`. Computing a boolean once before the loop
would stop it from seeing changes to the queue or cancellation state. For an `if let` or `while let` chain, a nested
check can use the variable introduced by the pattern. In a loop, preserve the original exit condition.

Braces alone do not hide an operator, but a block that binds and then tests a named boolean is fine. Closure bodies and
`for` iterators are not conditions. As with [`max_logical_operators`](max_logical_operators.md), operators written in a
macro argument can warn; operators introduced by the macro do not. The two rules are independent: a long inline
condition can trigger both.
