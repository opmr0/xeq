# Contributing

Contributions are welcome, bug fixes, new features, or doc improvements.

If you're planning something large, [open an issue](https://github.com/opmr0/xeq/issues) first so we can discuss it before you invest the time.

---

## Getting Started

```bash
git clone https://github.com/opmr0/xeq
cd xeq
cargo build
cargo test
```

---

## Making Changes

- Keep changes focused, one fix or feature per PR
- Match the existing code style
- If you're adding a feature, update the README too
- If you're fixing a bug, explain what caused it in the PR description

---

## Before Submitting

Run `xeq run submit` or manually:

```bash
cargo fmt
cargo clippy  # no warnings
cargo test    # all pass
```

---

## Project Structure

```
src/
  main.rs       # CLI parsing and command dispatch
  config.rs     # Path saving/loading and TOML reading
  runner.rs     # Script execution logic
  types.rs      # Shared types (Script, Scripts, Config, SavedPath)
  macros.rs     # log! and err! macros
  validation.rs # Validation logic for xeq validate
  templates.rs  # Template loading for xeq init
  templates/    # Built-in init templates
examples/       # Ready-to-use TOML files
```

---

## Reporting Bugs

Open an issue and include:

- What you ran
- What you expected
- What actually happened
- Your OS and xeq version (`xeq --version`)