# Changelog

All notable changes to xeq will be documented here.

---

## [v1.3.0] - 2025-03-10

### Added
- Script `options` array — set default flags per script in the TOML file
- `--parallel` flag — run all commands in a script concurrently
- `--allow-recursion` flag — opt into recursive script calls (circular dependency detection enabled by default)

### Changed
- Migrated script files from JSON to TOML format
- Refactored codebase into modules: `config.rs`, `runner.rs`, `types.rs`, `macros.rs`
- Grouped `run()` flags into a `RunOptions` struct
- Tests moved into their respective modules

### Fixed
- All clippy warnings (`io::Error::other`, `strip_prefix`, `needless_borrow`, `needless_question_mark`)

---

## [v1.2.0] - 2025-03-09

### Added
- `--args` flag — pass arguments to scripts and reference them with `{{1}}`, `{{2}}`, etc.

### Fixed
- Circular dependency detection for `xeq://` nested script calls

---

## [v1.1.0] - 2025-03-08

### Added
- Nested scripts via `xeq://script-name` syntax — call other scripts from within a script

---

## [v1.0.0] - 2025-03-08

Initial release.

### Added
- `xeq config <path>` — save path to your script file
- `xeq run <script>` — run a named script sequentially
- `xeq list` — list all scripts and their commands
- `--continue-on-err`, `--quiet`, `--clear` flags
- `cd` command support — changes working directory for subsequent commands
- Cross-platform support: Linux, macOS, Windows