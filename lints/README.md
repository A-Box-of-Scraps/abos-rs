# Dylint Lint Rules

A Box of Scraps's lint rules shared across projects.

## Installation

Install Dylint and its linker:

```sh
cargo install cargo-dylint dylint-link
```

Add the following to your workspace's root `Cargo.toml`. If you already have a `libraries` array under
`[workspace.metadata.dylint]`, append the entry to that array instead.

```toml
[workspace.metadata.dylint]
libraries = [
  { git = "https://github.com/A-Box-of-Scraps/abos-rs.git", pattern = "lints/rules" },
]
```

From the workspace root, run the configured lints:

```sh
cargo dylint --all
```

### Pin a commit

Add `rev` to the same library entry in your root `Cargo.toml`. Replace `<commit-sha>` with the full Git commit hash from
this repository:

```toml
[workspace.metadata.dylint]
libraries = [
  { git = "https://github.com/A-Box-of-Scraps/abos-rs.git", pattern = "lints/rules", rev = "<commit-sha>" },
]
```

### Select a single rule

The library contains all the rules. To report only `no_magic_numbers` from this library, add the following to your
workspace's `.cargo/config.toml`:

```toml
[env]
DYLINT_RUSTFLAGS = "-A abos_rules -W no_magic_numbers"
```

Keep the flags in this order: `-A abos_rules` silences the group, then `-W no_magic_numbers` enables warnings for that
rule. Use `-D` instead of `-W` to make violations errors, or replace `no_magic_numbers` with another name from the
[rule list](rules/docs/README.md). Run `cargo dylint --all` as before.

## Dylint Lints

Dylint lints documentation starts [HERE](rules/docs/README.md).

## License

Licensed under the [MIT License](LICENSE) by A Box of Scraps.
