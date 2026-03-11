<img src="./logo.png" alt="logo" width="120"/>

# xeq

[![Crates.io](https://img.shields.io/crates/v/xeq)](https://crates.io/crates/xeq)
[![Downloads](https://img.shields.io/crates/d/xeq)](https://crates.io/crates/xeq)
[![License](https://img.shields.io/crates/l/xeq)](LICENSE)
[![Build](https://github.com/opmr0/xeq/actions/workflows/release.yml/badge.svg)](https://github.com/opmr0/xeq/actions)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

**Run sequences of shell commands with a single word.**

Define your commands once in a `xeq.toml` file. Run them from anywhere, on any OS, without rewriting them every time.

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
- [TOML Format](#toml-format)
- [Features](#features)
  - [1. Script Options](#1-script-options)
  - [2. Variables](#2-variables)
  - [3. Arguments](#3-arguments)
  - [4. Environment Variables](#4-environment-variables)
  - [5. Nested Scripts](#5-nested-scripts)
  - [6. Parallel Execution](#6-parallel-execution)
  - [7. Local Configuration](#7-local-configuration)
  - [8. cd with Operators](#8-cd-with-operators)
- [Examples](#examples)
- [How It Works](#how-it-works)
- [Contributing](#contributing)
- [License](#license)

---

## Why xeq?

Every project has a setup ritual — install dependencies, build, run tests, configure things. You either memorize the steps, paste them from a notes file, or write a shell script that only works on your machine.

xeq gives you a better option:

- Write your commands once in a `xeq.toml` file
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
| `.env` loading         | ✅                    | ✅                  | ❌                 |
| Multi-language recipes | ❌                    | ✅                  | ❌                 |
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

### `xeq init`

Creates a starter `xeq.toml` in the current directory. Will not overwrite an existing file.

```bash
xeq init
```

---

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
xeq run build --continue-on-err
xeq run dev --quiet
xeq run test --parallel
xeq run create --args my-app
xeq run deploy --args env=prod
```

| Flag                 | Short | Description                                                 |
| -------------------- | ----- | ----------------------------------------------------------- |
| `--continue-on-err`  | `-C`  | Keep going even if a command fails                          |
| `--quiet`            | `-q`  | Hide xeq's own log messages                                 |
| `--clear`            | `-c`  | Clear the terminal before each command                      |
| `--parallel`         | `-p`  | Run all commands at the same time                           |
| `--allow-recursion`  |       | Let a script call itself                                    |
| `--args <values...>` | `-a`  | Pass arguments into the script — positional or `key=value`  |
| `--global`           | `-g`  | Use the globally saved path instead of the local `xeq.toml` |

---

### `xeq list`

Shows all scripts in your TOML file — their names, descriptions, and commands.

```bash
xeq list
xeq list --global
```

---

## TOML Format

A `xeq.toml` file contains named scripts. Each script needs at least a `run` array:

```toml
[my-script]
description = "What this script does"
options = ["quiet"]
run = [
    "command one",
    "command two"
]
```

- Script names are **case-sensitive** — `Build` and `build` are different scripts
- `description` is optional and only shows in `xeq list`
- `options` are optional — see [Script Options](#1-script-options)

---

## Features

### 1. Script Options

Bake default behavior into a script so you don't have to pass flags every time:

```toml
[build]
options = ["quiet", "parallel"]
run = ["cargo build", "cargo test"]
```

Now `xeq run build` always runs quietly and in parallel — no flags needed.

**Available options:** `quiet`, `clear`, `parallel`, `continue_on_err`, `allow_recursion`

**Toggling:** CLI flags _toggle_ script options. If a script has `quiet` set and you pass `--quiet`, it turns quiet _off_ for that run.

```bash
xeq run build          # quiet ON  (from TOML)
xeq run build --quiet  # quiet OFF (toggled off by CLI flag)
```

---

### 2. Variables

Use a `[vars]` block to define reusable values. Reference them in commands with `{{@varname}}`:

```toml
[vars]
image = "myapp:latest"
env = "development"

[build]
run = ["docker build -t {{@image}} ."]

[start]
run = ["APP_ENV={{@env}} npm start"]
```

**Local variables** let a specific script override a global value:

```toml
[vars]
image = "myapp:latest"

[build]
vars.image = "myapp:build"
run = ["docker build -t {{@image}} ."]   # uses "myapp:build"

[push]
run = ["docker push {{@image}}"]          # uses "myapp:latest"
```

**Override at runtime** using `--args`:

```bash
xeq run build --args image=myapp:hotfix
```

**Resolution order — most specific wins:**

```
--args (runtime)  →  local vars (per script)  →  global vars (file-level)
```

---

### 3. Arguments

For values that change every run, use positional placeholders `{{1}}`, `{{2}}`, etc.:

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

Mix named and positional args in a single call:

```bash
xeq run deploy --args env=production my-app
```

---

### 4. Environment Variables

Reference system environment variables in commands using `{{$VARNAME}}`:

```toml
[deploy]
run = ["deploy --token {{$API_TOKEN}} --env {{$DEPLOY_ENV}}"]
```

xeq also automatically loads a `.env` file from the current directory if one exists, so your variables are available without setting them manually.

```bash
# .env
API_TOKEN=abc123
DEPLOY_ENV=production
```

```bash
xeq run deploy   # API_TOKEN and DEPLOY_ENV are loaded automatically
```

---

### 5. Nested Scripts

A script can call other scripts using the `xeq://` prefix:

```toml
[install]
run = ["npm install"]

[build]
run = ["npm run build"]

[deploy]
run = [
    "xeq://install",
    "xeq://build",
    "npm run deploy"
]
```

Running `xeq run deploy` automatically runs `install` and `build` first, in order.

> **Circular dependency protection:** If a script tries to call itself directly or through a chain, xeq exits with an error. Add `allow_recursion` to `options` if you intentionally need this.

---

### 6. Parallel Execution

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
xeq run check      # all three run at the same time
xeq run check -p   # same using CLI flag
```

> Scripts with `cd` commands or `xeq://` calls cannot run in parallel — xeq will exit with a clear error if you try.

---

### 7. Local Configuration

xeq automatically detects a `xeq.toml` in your current directory. If one exists, it uses that. If not, it falls back to the globally saved path.

This means in most projects you never need to run `xeq config` at all — just put the file in the project root.

To force the global path, pass `--global`:

```bash
xeq run setup --global
xeq list --global
```

---

### 8. cd with Operators

xeq supports shell-style operators after `cd`, so you can chain commands naturally:

```toml
[setup]
run = [
    "cd my-app && npm install",
    "cd /tmp || echo 'fallback'",
    "cd build; echo always runs",
    "cd server & echo background"
]
```

| Operator | Behavior                                |
| -------- | --------------------------------------- |
| `&&`     | Run next command only if `cd` succeeded |
| `\|\|`   | Run next command only if `cd` failed    |
| `;`      | Always run next command                 |
| `&`      | Spawn next command in background        |
| `!`      | Negate the result of `cd`               |

---

## Example Files

## The [`examples/`](./examples) folder has ready-to-use TOML files for common workflows.

| File | Description | Key Features Used |
|---|---|---|
| `react-tailwind.toml` | Scaffold and run a React + Tailwind project | variables, cd operators |
| `nextjs.toml` | Next.js project setup and pipeline | nested scripts, variables |
| `rust-project.toml` | Rust checks, build and publish | parallel, nested scripts |
| `docker.toml` | Docker image and container management | variables, nested scripts |
| `git-workflow.toml` | Common git operations | variables, arguments |
| `nested-scripts.toml` | CI pipeline from reusable pieces | nested scripts |
| `env-vars.toml` | Deploy and notify using env vars | `{{$VAR}}`, nested scripts |
| `python-project.toml` | Virtualenv, checks and PyPI publish | parallel, nested scripts, variables |
| `database.toml` | Migrations, seed, dump and restore | env vars, arguments, nested scripts |
| `monorepo.toml` | Multi-package frontend workspace | parallel, variables, nested scripts |
| `aws-deploy.toml` | ECR push and ECS deploy pipeline | env vars, nested scripts |
| `go-project.toml` | Go build, test and cross-compile | parallel, nested scripts, variables |
| `arguments.toml` | Positional and named arg patterns | arguments, variables |
| `script-options.toml` | Flag toggle mechanic demonstrations | options, parallel, quiet, continue_on_err |

---

## How It Works

- xeq stores your TOML file path using the system config directory
- Commands run through `sh -c` on Linux/macOS and `cmd /C` on Windows
- `cd` commands update the working directory for all subsequent commands in that script
- Variables resolve in order: `--args` → local vars → global vars
- Environment variables are loaded from `.env` automatically before any script runs
- On failure, xeq exits with the same exit code as the failed command
- Script names are case-sensitive: `Build` and `build` are different scripts

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

---

## License

MIT — [LICENSE](LICENSE)
