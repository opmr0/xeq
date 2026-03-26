## Contributing

Contributions are welcome, whether it's a bug fix, a new feature, or an improvement to the docs. [Open an issue](https://github.com/opmr0/xeq/issues).

**Getting started:**

```bash
git clone https://github.com/opmr0/xeq
cd xeq
cargo build
cargo test
```

**Before submitting a PR:**

- Run `cargo fmt` to format your code
- Run `cargo clippy` and fix any warnings
- Run `cargo test` and make sure all tests pass
- If you're adding a new feature, add tests for it

or run:
```bash
xeq run submit
```
**Project structure:**

```
src/
  main.rs       # CLI parsing and command dispatch
  config.rs     # Path saving/loading and TOML reading
  runner.rs     # Script execution logic
  types.rs      # Shared types (Script, Scripts, Config, SavedPath)
  macros.rs     # log! and err! macros
  validation.rs # Validation functions for `xeq validate`
  templates.rs  # for importing templates from /templates
  /templates    # templates for `xeq init <template>`
examples/       # Ready-to-use TOML files
```

---