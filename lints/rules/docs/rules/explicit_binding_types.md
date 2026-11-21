# explicit_binding_types

Warns when a simple `let` or `let mut` binding needs a type annotation because its initializer does not show the type.
A written annotation may use a type alias or an inferred lifetime (`'_`), but not a type or const placeholder (`_`).

## Accepted

```rust
let player: ServerPlayer = server.get_player();
let enabled = true;
let offsets = [-1, 0, 1];
let player = ServerPlayer::new();
let players = Vec::<ServerPlayer>::new();
```

Literals, casts, and explicit type ascriptions show the type. References, negation, tuples, and arrays qualify when their
contents do. A struct literal, brace-form enum variant, or `Type::new(...)` qualifies when it names the resulting struct
or enum. `Self` counts; a type alias in the initializer does not. For generic types, write at least one type or const
argument and all non-default arguments. Omitted arguments must resolve to their declared defaults.

## Rejected

```rust
let player = server.get_player();
let score = player.score;
let copy = score;
let mut players = Vec::new();
players.push(player);
let others: Vec<_> = Vec::<ServerPlayer>::new();
```

Annotate the first four bindings, or write `Vec::<ServerPlayer>::new()` for the fourth. For `others`, either write
`Vec<ServerPlayer>` or remove the annotation: its initializer already shows the full type. Nested placeholders such as
`Vec<Option<_>>` are also insufficient.

The rule checks the initializer as written: a macro call does not show its expansion’s type. It skips generated bindings,
destructuring, discards, uninitialized bindings, and results containing closures or opaque `impl Trait` types. It does
not check `if let`, `while let`, match arms, or loop patterns. Use `#[allow(explicit_binding_types)]` for exceptions.
