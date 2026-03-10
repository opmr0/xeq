<br/>

<img src="./logo.png" alt="logo" width="120"/>

# xeq

[![Crates.io](https://img.shields.io/crates/v/xeq)](https://crates.io/crates/xeq)
[![Downloads](https://img.shields.io/crates/d/xeq)](https://crates.io/crates/xeq)
[![License](https://img.shields.io/crates/l/xeq)](LICENSE)
[![Build](https://github.com/opmr0/xeq/actions/workflows/release.yml/badge.svg)](https://github.com/opmr0/xeq/actions)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

**Run sequences of shell commands with a single word.**

Define your commands once in a TOML file. Run them from anywhere, on any OS, without rewriting them every time.

```bash
xeq run setup
```

---

## Table of Contents

- [Why xeq?](#why-xeq)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [TOML File Format](#toml-file-format)
- [Features](#features)
  - [Script Options](#script-options)
  - [Nested Scripts](#nested-scripts)
  - [Arguments](#arguments)
  - [Parallel Execution](#parallel-execution)
- [How It Works](#how-it-works)
- [Examples](#examples)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Why xeq?

Every project has a setup ritual — install dependencies, build, run tests, configure things. You either memorize the steps, paste them from a notes file, or write a shell script that only works on your machine.

xeq gives you a better option. Write your commands in a `xeq.toml` file, commit it to your repo, and anyone on the team can run the exact same steps with one command — on Linux, macOS, or Windows.

---

## Installation

**macOS / Linux**
```bash
curl -sSf https://raw.githubusercontent.com/opmr0/xeq/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
iwr https://raw.githubusercontent.com/opmr0/xeq/main/install.ps1 -UseBasicParsing | iex
```

**Via cargo**
```bash
cargo install xeq
```

---

## Quick Start

**1. Create a `xeq.toml` in your project:**

```toml
[setup]
run = [
    "npm install",
    "npm run build"
]

[dev]
run = ["npm run dev"]
```

**2. Tell xeq where the file is (one time only):**
```bash
xeq config ./xeq.toml
```

**3. Run any script by name:**
```bash
xeq run setup
xeq run dev
```

That's it. xeq remembers the file path — you don't need to pass it every time.

---

## Commands

### `xeq config [path]`

Point xeq at your TOML file. This is saved globally so you only need to do it once per project (or when you move the file).

```bash
xeq config ./xeq.toml        # save the path
xeq config                    # open the saved file in your default editor
```

---

### `xeq run <script> [flags]`

Run a script from your TOML file. Commands in the script run one at a time in order. If a command fails, xeq stops — unless you use `--continue-on-err`.

```bash
xeq run setup
xeq run build --continue-on-err
xeq run dev --quiet
xeq run test --parallel
xeq run greet --args Alice 30
```

**Available flags:**

| Flag | Short | Description |
|------|-------|-------------|
| `--continue-on-err` | `-C` | Keep going even if a command fails |
| `--quiet` | `-q` | Hide xeq's own log messages, only show command output |
| `--clear` | `-c` | Clear the terminal before each command |
| `--parallel` | `-p` | Run all commands at the same time instead of one by one |
| `--allow-recursion` | | Let a script call itself (disabled by default to prevent infinite loops) |
| `--args <values...>` | `-a` | Pass arguments into the script |

---

### `xeq list`

Show all scripts in your TOML file along with their commands and options.

```bash
xeq list
```

```
build runs: --- options: ["quiet", "parallel"]
    cargo fmt
    cargo clippy
    cargo build --release
```

---

## TOML File Format

A `xeq.toml` file is a list of named scripts. Each script has a `run` array of shell commands:

```toml
[script-name]
run = [
    "command one",
    "command two",
    "command three"
]
```

Scripts are case-sensitive. You can define as many as you want in a single file.

---

## Features

### Script Options

Scripts can have default options baked in, so you don't have to pass flags every time. Add an `options` array to any script:

```toml
[build]
options = ["quiet", "parallel"]
run = ["cargo build", "cargo test"]
```

Now `xeq run build` always runs quietly and in parallel — no flags needed.

**Available options:** `quiet`, `clear`, `parallel`, `continue_on_err`, `allow_recursion`

**Toggling:** CLI flags *toggle* options. If `quiet` is set in TOML and you pass `--quiet` on the CLI, it turns quiet *off* for that run. This lets you override script defaults without editing the file.

```bash
xeq run build --quiet    # quiet is ON in TOML, this toggles it OFF
```

---

### Nested Scripts

Call other scripts from within a script using the `xeq://` prefix:

```toml
[install]
run = ["npm install"]

[build]
run = ["npm run build"]

[deploy]
run = [
    "xeq://install",     # runs the install script first
    "xeq://build",       # then the build script
    "npm run deploy"     # then this command
]
```

Running `xeq run deploy` automatically runs `install` and `build` first.

> **Circular dependency protection:** xeq tracks which scripts are currently running. If a script tries to call itself (directly or through a chain), xeq exits with an error. Pass `--allow-recursion` or add it to `options` if you intentionally want recursive behavior.

---

### Arguments

Reference arguments in commands using `{{1}}`, `{{2}}`, etc., then pass them with `--args`:

```toml
[create]
run = [
    "npm create vite@latest {{1}} -- --template react",
    "cd {{1}}",
    "npm install"
]
```

```bash
xeq run create --args my-app
```

This creates a new Vite project called `my-app`, changes into it, and installs dependencies.

> If a script uses `{{placeholders}}` but no `--args` are provided, xeq exits with an error instead of passing the raw placeholder to the shell.

---

### Parallel Execution

Run all commands in a script simultaneously instead of one by one:

```toml
[check]
options = ["parallel"]
run = [
    "cargo test",
    "cargo clippy",
    "cargo fmt --check"
]
```

```bash
xeq run check        # all three run at the same time
xeq run check -p     # same, using the CLI flag
```

> **Note:** In parallel mode, `cd` commands and `xeq://` calls are skipped — only plain shell commands are spawned as parallel threads.

---

## How It Works

- xeq stores your TOML file path globally using the system config directory
- Commands run through `sh -c` on Linux/macOS and `cmd /C` on Windows
- `cd` commands update the working directory for all subsequent commands in that script
- On failure, xeq exits with the same exit code as the failed command
- Script names are case-sensitive: `Build` and `build` are different scripts

---

## Examples

The [`examples/`](./examples) folder has ready-to-use TOML files for common workflows:

| File | What it does |
|------|-------------|
| `react-tailwind.toml` | Scaffold a React + Tailwind CSS project |
| `nextjs.toml` | Set up a Next.js project with TypeScript |
| `rust-project.toml` | Format, lint, test, and release a Rust project |
| `docker-app.toml` | Start, stop, rebuild, and tail logs for Docker Compose |
| `git-workflow.toml` | Sync, push, and stash/pop git operations |
| `scripts-nesting.toml` | Example of calling scripts from within scripts |

---

## Contributing

Contributions are welcome — whether it's a bug fix, a new feature, or an improvement to the docs [Open an issue](https://github.com/opmr0/xeq/issues).

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

**Project structure:**
```
src/
  main.rs       # CLI parsing and command dispatch
  config.rs     # Path saving/loading and TOML reading
  runner.rs     # Script execution logic
  types.rs      # Shared types (Script, Scripts, SavedPath)
  macros.rs     # log! and err! macros
examples/       # Ready-to-use TOML files
```

If you're unsure whether a change fits the project, open an issue first to discuss it before writing code.

---

## License

MIT — [LICENSE](LICENSE)
