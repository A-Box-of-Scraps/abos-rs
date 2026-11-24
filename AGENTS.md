# abos-rs

Rust utilities and lint rules shared across projects.

## Project Structure

Here is an overview of the project:

```
.cargo/
.githooks/
.github/
lints/
  .cargo/
  rules/
    docs/
    src/
    tests/
    Cargo.toml
  Cargo.lock
  Cargo.toml
  LICENSE
  README.md
  rust-toolchain.toml
packages/
scripts/
target/
.gitattributes
.gitignore
.jscpd.json
AGENTS.md
Cargo.lock
Cargo.toml
clippy.toml
dprint.json
LICENSE
mise.toml
README.md
ruff.toml
rust-toolchain.toml
rustfmt.toml
```

## Verification

At least `mise run check` must pass to validate changes.

## Commits & Pull Requests

- Follow the Conventional Commits specification for commit messages.
- Pull request summaries should include the related issue(s), a brief description of the changes, and how the changes
  were tested.
