# Changelog

All notable changes to xeq will be documented here.

---

## 2.2.0 - 2026-03-26

### Added

- add `xeq toml` command to view the toml format
- templates for `xeq init <templates>`
- shell field per file to chose witch shell to use from sh, zsh, bash, fish, cmd or powershell and the default for windows is cmd and for any other OS is sh

## 2.1.0 - 2026-03-25

## Added

- `--allow-empty-vars` option/flag: makes arguments optional; if not provided, placeholders `{{@varname}}` or `{{n}}` are passed through as literal strings.
- parallel_threads field in xeq.toml or global config: enables parallel execution and specifies the number of threads per command.
- `xeq validate` now checks for invalid parallel execution configurations (including 1-threaded runs).
- `xeq validate` now checks for continue_on_err option with a fallback script
- Command aliases: `xeq run` to `xeq r`, `xeq config` → `xeq c`, etc.
- Added descriptions for commands and flags in help output.
- fallback field: allows a script to specify another script to run automatically if it fails.

## Changed

- Arguments are required by default; use `--allow-empty-vars` to make them optional.
- Replaced the old parallel option with `parallel_threads` field to avoid spawning a separate thread per command.

---

## v2.0.1 - 2026-03-20

### Changed

- update dependencies

---

## v2.0.0 - 2026-03-14

### Changed

- runner now returns Result instead of calling process::exit directly
- parallel validation check moved outside the command loop
- explicit imports replacing glob imports in main.rs

### Fixed

- recursion errors in validation no longer incorrectly show "passed"
- background processes correctly marked as intentional
- canonicalize error in save_path now propagates instead of panicking

---

## v1.8.0 - 2026-03-13

- Commands now show execution time after completing
- Added `--summary` flag to `xeq run` - prints a table of all commands and their execution times after the script finishes

---

## v1.7.0 -

### Added

- `--no-env` flag to `xeq run` — skips `.env` file loading for that run
- `dir` field to scripts — sets the working directory without needing a `cd` command
- `xeq validate` command — checks all scripts for errors without running them

---

## v1.6.0 - 2026-03-12

### Added

- **Environment variable interpolation** — use `{{$VARNAME}}` in any command to inject a system environment variable at runtime
- **Automatic `.env` loading** — xeq now loads a `.env` file from the current directory automatically before any script runs, no manual setup needed
- **`cd` operator support** — `cd` commands now support `&&`, `||`, `;`, `&`, and `!` operators instead of throwing an error
  - `cd foo && bar` — run `bar` only if `cd` succeeded
  - `cd foo || bar` — run `bar` only if `cd` failed
  - `cd foo; bar` — always run `bar` regardless
  - `cd foo & bar` — spawn `bar` in the background
  - `! cd foo` — negate the result of `cd`

### Fixed

- **Parallel mode variable substitution** — variables (`{{@var}}`), positional args (`{{1}}`), and environment vars (`{{$VAR}}`) were not being resolved before commands were spawned in parallel mode. All substitution now happens before threads are created
- **Parallel validation inside loop** — the `cd` and `xeq://` checks were running on every loop iteration instead of once before the loop, causing redundant work and fragile logic
- **Operator precedence bug on `cd` validation** — `line.starts_with("cd ") && line.contains("&&") || line.contains(";")` was incorrectly matching any line containing `;` due to missing parentheses. This check has been removed entirely in favor of graceful handling

### Changed

- Added warning message when `current_dir()` fails and xeq falls back to `"."`

---

## v1.5.0 - 2026-03-11

### Added

- **Variables system** — define reusable values in a `[vars]` block and reference them in commands with `{{@varname}}`
- **Local variables** — scripts can define their own vars with `vars.key = "value"` to override global ones for that script only
- **Named arguments** — `--args` now accepts `key=value` pairs in addition to positional values, allowing named overrides of variables at runtime
- **Variable resolution order** — `--args` (runtime) -> local vars (per script) -> global vars (file-level). Most specific value always wins.

### Changed

- `--args` now supports both positional (`{{1}}`) and named (`{{@varname}}`) placeholders in a single call
- Improved error message when a `{{@varname}}` placeholder has no value defined at any level

### Fixed

- fix some bugs with the `cd` command handling

---

## v1.4.0 - 2026-03-11

### Added

- `--global` / `-g` flag for `xeq run` and `xeq list` — explicitly use the saved global path instead of the local file
- Local `xeq.toml` auto-detection — xeq now looks for a `xeq.toml` in the current directory first, falling back to the saved path
- `xeq init` creates a new file named `xeq.toml`

### Changed

- Script `options` now use a typed enum instead of raw strings — unknown options will error at parse time instead of silently doing nothing
- Parallel mode now warns when `cd` or `xeq://` commands are skipped instead of silently ignoring them

---

## v1.3.0 - 2025-03-10

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

## v1.2.0 - 2025-03-09

### Added

- `--args` flag — pass arguments to scripts and reference them with `{{1}}`, `{{2}}`, etc.

### Fixed

- Circular dependency detection for `xeq://` nested script calls

---

## v1.1.0 - 2025-03-08

### Added

- Nested scripts via `xeq://script-name` syntax — call other scripts from within a script

---

## v1.0.0 - 2025-03-08

Initial release.

### Added

- `xeq config <path>` — save path to your script file
- `xeq run <script>` — run a named script sequentially
- `xeq list` — list all scripts and their commands
- `--continue-on-err`, `--quiet`, `--clear` flags
- `cd` command support — changes working directory for subsequent commands
- Cross-platform support: Linux, macOS, Windows
