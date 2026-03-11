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
- [How does xeq compare?](#how-does-xeq-compare)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [TOML File Format](#toml-file-format)
- [Features](#features)
  - [Script Options](#script-options)
  - [Nested Scripts](#nested-scripts)
  - [Local Configuration](#local-configuration)
  - [Variables](#variables)
  - [Arguments](#arguments)
  - [Parallel Execution](#parallel-execution)
- [Real-World Examples](#real-world-examples)
- [How It Works](#how-it-works)
- [Example Files](#example-files)
- [Contributing](#contributing)
- [License](#license)

---

## Why xeq?

Every project has a setup ritual — install dependencies, build, run tests, configure things. You either memorize the steps, paste them from a notes file, or write a shell script that only works on your machine.

xeq gives you a better option:

- Write your commands in a `xeq.toml` file
- Commit it to your repo
- Anyone on the team runs the exact same steps with one command — on Linux, macOS, or Windows
---

## How does xeq compare?

| Feature                | xeq                   | just                | make               |
| ---------------------- | --------------------- | ------------------- | ------------------ |
| Config format          | TOML                  | Custom syntax       | Makefile syntax    |
| Cross-platform         | ✅                    | ✅                  | ⚠️ poor on Windows |
| Parallel execution     | ✅ built-in           | ❌                  | ❌                 |
| Nested scripts         | ✅ (`xeq://`)         | ✅                  | ⚠️                 |
| Variables              | ✅ global + local     | ✅                  | ✅                 |
| Argument passing       | ✅ named + positional | ✅ named + defaults | ⚠️                 |
| Flag toggle mechanic   | ✅                    | ❌                  | ❌                 |
| Multi-language recipes | ❌                    | ✅                  | ❌                 |
| `.env` loading         | ❌                    | ✅                  | ❌                 |
| Learning curve         | None (TOML)           | Low (new syntax)    | High               |

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

**1. Create a `xeq.toml` in your project root:**

```toml
[setup]
run = [
    "npm install",
    "npm run build"
]

[dev]
run = ["npm run dev"]
```

**2. Point xeq at the file (one time only):**

```bash
xeq config ./xeq.toml
```

> If there is a `xeq.toml` in your current directory, xeq finds it automatically — no config step needed.

**3. Run any script by name:**

```bash
xeq run setup
xeq run dev
```

That's it. From now on, anyone who clones the repo can run `xeq run setup` and get the exact same result.

---

## Commands

### `xeq config [path]`

Saves the path to your TOML file globally. You only need to do this once, or when you move the file.

```bash
xeq config ./xeq.toml        # save the path
xeq config                    # open the saved file in your default editor
```

---

### `xeq run <script> [flags]`

Runs a script by name. Commands execute one at a time in order. If any command fails, xeq stops — unless you pass `--continue-on-err`.

```bash
xeq run setup
xeq run build --continue-on-err   # keep going even if something fails
xeq run dev --quiet               # hide xeq's own output
xeq run test --parallel           # run all commands at the same time
xeq run create --args my-app      # pass a positional argument
xeq run deploy --args env=prod    # pass a named argument
```

**Available flags:**

| Flag                 | Short | Description                                                              |
| -------------------- | ----- | ------------------------------------------------------------------------ |
| `--continue-on-err`  | `-C`  | Keep going even if a command fails                                       |
| `--quiet`            | `-q`  | Hide xeq's own log messages, only show command output                    |
| `--clear`            | `-c`  | Clear the terminal before each command runs                              |
| `--parallel`         | `-p`  | Run all commands at the same time instead of one by one                  |
| `--allow-recursion`  |       | Let a script call itself (disabled by default to prevent infinite loops) |
| `--args <values...>` | `-a`  | Pass arguments into the script — positional or `key=value`               |
| `--global`           | `-g`  | Use the globally saved path instead of the local `xeq.toml`             |

---

### `xeq init`

Creates a starter `xeq.toml` in the current directory with a sample script. Will not overwrite an existing file.

```bash
xeq init
```

---

### `xeq list`

Shows all scripts in your TOML file — their names, descriptions, and commands.

```bash
xeq list
```

```
build --- Format, lint and release
 runs:
    cargo fmt
    cargo clippy
    cargo build --release
```

---

## TOML File Format

A `xeq.toml` file contains named scripts. Each script needs at least a `run` array:

```toml
[my-script]
description = "What this script does"
options = ["quiet"]
run = [
    "command one",
    "command two",
    "command three"
]
```

- Script names are **case-sensitive** — `Build` and `build` are different scripts
- `description` is optional and only shows in `xeq list`
- `options` are optional — see [Script Options](#script-options)
- You can define as many scripts as you want in a single file

---

> **Note:** `cd` must be on its own line. Using `cd` with `&&` or `;` is not supported and will cause an error.

## Features

### Script Options

Bake default behavior into a script so you don't have to pass flags every time:

```toml
[build]
options = ["quiet", "parallel"]
run = ["cargo build", "cargo test"]
```

Now `xeq run build` always runs quietly and in parallel — no flags needed.

**Available options:** `quiet`, `clear`, `parallel`, `continue_on_err`, `allow_recursion`

> Invalid options cause a parse error before any commands run.

**Toggling:** CLI flags _toggle_ script options. If a script has `quiet` set and you pass `--quiet`, it turns quiet _off_ for that run. This lets you override defaults without editing the file.

```bash
xeq run build          # quiet ON  (from TOML)
xeq run build --quiet  # quiet OFF (toggled by CLI flag)
```

---

### Nested Scripts

A script can call other scripts using the `xeq://` prefix. This lets you build bigger workflows out of smaller reusable pieces:

```toml
[install]
run = ["npm install"]

[build]
run = ["npm run build"]

[deploy]
run = [
    "xeq://install",   # runs install first
    "xeq://build",     # then build
    "npm run deploy"   # then this
]
```

Running `xeq run deploy` automatically runs `install` and `build` first, in order.

> **Circular dependency protection:** If a script tries to call itself (directly or through a chain), xeq exits with an error. Add `allow_recursion` to `options` if you intentionally need recursive behavior.

---

### Local Configuration

xeq automatically detects a `xeq.toml` in your current directory. If one exists, it uses that. If not, it falls back to the globally saved path.

This means in most projects you never need to run `xeq config` at all — just put the file in the project root.

To force xeq to use the global path, pass `--global`:

```bash
xeq run setup --global
xeq list --global
```

---

### Variables

Use a `[vars]` block to define reusable values at the top of your file. Reference them in commands with `{{@varname}}`:

```toml
[vars]
image = "myapp:latest"
env = "development"

[build]
run = ["docker build -t {{@image}} ."]

[start]
run = ["APP_ENV={{@env}} npm start"]
```

Both scripts share the same values. Change them in one place and every script picks up the update.

**Local variables** let a specific script use a different value without affecting others:

```toml
[vars]
image = "myapp:latest"       # default for all scripts

[build]
vars.image = "myapp:build"   # only applies to this script
run = ["docker build -t {{@image}} ."]

[push]
run = ["docker push {{@image}}"]   # still uses "myapp:latest"
```

**Override at runtime** using a named `--arg` — useful for one-off runs without editing the file:

```bash
xeq run build --args image=myapp:hotfix
```

**Resolution order — most specific wins:**

```
--args (runtime)  ->  local vars (per script)  ->  global vars (file-level)
```

> Using `{{@varname}}` with no value defined at any level causes a clear error before the command runs.

---

### Arguments

For values that change every run, use positional placeholders `{{1}}`, `{{2}}`, etc. and pass them with `--args`:

```toml
[create]
run = [
    "npm create vite@latest {{1}} -- --template {{2}}",
    "cd {{1}}",
    "npm install"
]
```

```bash
xeq run create --args my-app react
# {{1}} = my-app
# {{2}} = react
```

You can mix named and positional args in a single call:

```bash
xeq run deploy --args env=production my-app
```

> If a script uses placeholders but no `--args` are given, xeq passes the raw `{{1}}` to the shell.

---

### Parallel Execution

Run all commands in a script at the same time instead of one by one:

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
xeq run check     # all three run simultaneously
xeq run check -p  # same using the CLI flag
```

> Scripts with `cd` commands or `xeq://` calls cannot run in parallel — xeq will exit with a clear error if you try.

---


## How It Works

- xeq stores your TOML file path using the system config directory
- Commands run through `sh -c` on Linux/macOS and `cmd /C` on Windows
- `cd` commands update the working directory for all subsequent commands in that script
- Variables resolve in order: `--args` → local vars → global vars
- On failure, xeq exits with the same exit code as the failed command
- Script names are case-sensitive: `Build` and `build` are different scripts

---

## Example Files

The [`examples/`](./examples) folder has ready-to-use TOML files for common workflows:

| File                   | What it does                                           |
| ---------------------- | ------------------------------------------------------ |
| `react-tailwind.toml`  | Scaffold a React + Tailwind CSS project                |
| `nextjs.toml`          | Set up a Next.js project with TypeScript               |
| `rust-project.toml`    | Format, lint, test, and release a Rust project         |
| `docker-app.toml`      | Start, stop, rebuild, and tail logs for Docker Compose |
| `git-workflow.toml`    | Sync, push, and stash/pop git operations               |
| `scripts-nesting.toml` | Example of calling scripts from within scripts         |

---

## Contributing

Contributions are welcome — whether it's a bug fix, a new feature, or an improvement to the docs. [Open an issue](https://github.com/opmr0/xeq/issues).

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
  types.rs      # Shared types (Script, Scripts, Config, SavedPath)
  macros.rs     # log! and err! macros
examples/       # Ready-to-use TOML files
```

If you're unsure whether a change fits the project, open an issue first.

---

## License

MIT — [LICENSE](LICENSE)