<div align="center">
<img src="./assets/logo_rounded.png" alt="logo" width="150"/>

# xeq

[![Crates.io](https://img.shields.io/crates/v/xeq)](https://crates.io/crates/xeq)
[![Downloads](https://img.shields.io/crates/d/xeq)](https://crates.io/crates/xeq)
[![License](https://img.shields.io/crates/l/xeq)](LICENSE)
[![Build](https://github.com/opmr0/xeq/actions/workflows/release.yml/badge.svg)](https://github.com/opmr0/xeq/actions)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

**xeq runs sequences of commands from a single TOML file, making repetitive tasks fast and consistent.**

Every project has a setup ritual. Ten commands, always in the same order, every time. Write them once in a `xeq.toml`, commit it, and anyone on any OS runs the exact same steps with one command.

</div>

---

## Table of Contents

- [Demo](#demo)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Comparison](#comparison)
- [Commands](#commands)
- [TOML Format](#toml-format)
- [Features](#features)
  - [1. Script Options](#1-script-options)
  - [2. Variables](#2-variables)
  - [3. Arguments](#3-arguments)
  - [4. Environment Variables](#4-environment-variables)
  - [5. Nested Scripts](#5-nested-scripts)
  - [6. Parallel Execution](#6-parallel-execution)
  - [7. Global Configuration](#7-global-configuration)
  - [8. Run Summary](#8-run-summary)
  - [9. Events](#9-events)
  - [10. Custom Shells](#10-custom-shells)
- [Examples](#examples)
- [How It Works](#how-it-works)
- [License](#license)

---

## Demo

![demo](./assets/demo.gif)

## Installation

**macOS / Linux**

```bash
curl -sSf https://raw.githubusercontent.com/opmr0/xeq/main/install.sh | sh
```

**Windows (Powershell)**

```powershell
iwr https://raw.githubusercontent.com/opmr0/xeq/main/install.ps1 -UseBasicParsing | iex
```

**Via cargo (Rust package manager)**

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

**2. Run any script by name:**

```bash
xeq run setup
xeq run dev
```

xeq finds `xeq.toml` in the current directory automatically. Use `xeq init` to generate a starter file.

---

## Comparison

| Feature            | **xeq**              | **Makefile**       | **npm scripts** | **just**     |
| ------------------ | -------------------- | ------------------ | --------------- | ------------ |
| File type          | TOML                 | Makefile           | package.json    | Justfile     |
| Cross-platform     | Yes                  | Mostly Linux/macOS | Yes             | Yes          |
| Validation command | Yes (`xeq validate`) | No                 | No              | No           |
| Variables          | Yes                  | Yes                | Limited         | Yes          |
| Args support       | Yes                  | Limited            | Limited         | Yes          |
| Nested scripts     | Yes (`xeq:`)         | Yes                | No              | Yes          |
| Parallel execution | Yes                  | Yes (`-j`)         | No              | No           |
| .env support       | Yes (auto)           | No                 | No              | Yes (opt-in) |
| Init templates     | Yes (30+)            | No                 | No              | No           |

---

## Commands

### `xeq run <script> [flags]`

Runs a script by name. Commands execute in order. If any command fails, xeq stops unless you pass `--continue-on-err`.

```bash
xeq run setup
xeq run build --continue-on-err
xeq run dev --quiet
xeq run test --parallel
xeq run create --args my-app
xeq run deploy --args env=prod
```

| Flag                       | Short | Description                                                       |
| -------------------------- | ----- | ----------------------------------------------------------------- |
| `--continue-on-err`        | `-C`  | Keep going even if a command fails                                |
| `--quiet`                  | `-q`  | Hide xeq's own log messages                                       |
| `--clear`                  | `-c`  | Clear the terminal before each command                            |
| `--parallel [threads]`     | `-p`  | Run all commands in parallel (default: logical CPU count)         |
| `--args <values...>`       | `-a`  | Pass arguments into the script, positional or `key=value`         |
| `--global`                 | `-g`  | Use the globally saved `xeq.toml` instead of the local one       |
| `--summary`                | `-s`  | Print a timing summary after the script finishes                  |
| `--dry-run`                | `-d`  | Preview commands without executing them                           |
| `--no-events`              | `-e`  | Disable events for this run                                       |
| `--allow-empty-args`       | `-A`  | Skip errors for missing variables or arguments                    |
| `--no-env`                 |       | Skip loading the `.env` file                                      |
| `--allow-recursion`        |       | Let a script call itself                                          |

---

### `xeq init [template]`

Creates a starter `xeq.toml` in the current directory. Will not overwrite an existing file.

```bash
xeq init
xeq init rust
xeq init docker
```

**Available templates:** `android`, `ansible`, `astro`, `aws`, `bun`, `deno`, `django`, `docker`, `dotnet`, `elixir`, `expo`, `fastapi`, `flutter`, `git`, `go`, `hugo`, `java-gradle`, `java-maven`, `kubernetes`, `laravel`, `monorepo`, `nestjs`, `nextjs`, `node`, `python`, `rails`, `react`, `rust`, `svelte`, `tauri`

---

### `xeq validate`

Checks all scripts for errors without running anything.

```bash
xeq validate
xeq validate --global
```

Catches undefined variables, missing nested scripts, circular dependencies, invalid shells, parallel conflicts, and more.

---

### `xeq list`

Shows all scripts in your `xeq.toml` - names, descriptions, and commands.

```bash
xeq list
xeq list --global
```

---

### `xeq config [path]`

Saves a `xeq.toml` path globally so you can run it from anywhere.

```bash
xeq config ~/my-scripts/xeq.toml   # save once
xeq config                          # open saved file in your editor
```

---

### `xeq toml`

Prints the full TOML format reference.

---

### Aliases

| Command        | Alias   |
| -------------- | ------- |
| `xeq run`      | `xeq r` |
| `xeq config`   | `xeq c` |
| `xeq init`     | `xeq i` |
| `xeq validate` | `xeq v` |

---

## TOML Format

Each script needs at minimum a `run` array:

```toml
[my-script]
description = "What this script does"
dir = "./my_app"
parallel_threads = 4
options = ["quiet"]
run = [
    "command one",
    "command two"
]
```

- `run` - required, commands to execute in order
- `description` - optional, shown in `xeq list`
- `dir` - optional, working directory (absolute or relative)
- `parallel_threads` - optional, enables parallel execution with a set thread count
- `options` - optional, baked-in flags. see [Script Options](#1-script-options)

Script names are case-sensitive. `Build` and `build` are different scripts.

---

## Features

### 1. Script Options

Bake default behavior into a script so you don't have to pass flags every time:

```toml
[build]
options = ["quiet", "continue_on_err"]
run = ["cargo build", "cargo test"]
```

**Available options:** `quiet`, `clear`, `continue_on_err`, `allow_recursion`, `summary`, `allow_empty_vars`

CLI flags toggle script options. If `quiet` is baked in and you pass `--quiet`, it turns quiet off for that run.

```bash
xeq run build          # quiet ON  (from TOML)
xeq run build --quiet  # quiet OFF (toggled)
```

---

### 2. Variables

Define reusable values in a `[vars]` block and reference them with `{{@varname}}`:

```toml
[vars]
image = "myapp:latest"
env = "development"

[build]
run = ["docker build -t {{@image}} ."]

[start]
run = ["APP_ENV={{@env}} npm start"]
```

**Local variables** override global ones for a specific script:

```toml
[vars]
image = "myapp:latest"

[build]
vars.image = "myapp:build"
run = ["docker build -t {{@image}} ."]   # uses "myapp:build"

[push]
run = ["docker push {{@image}}"]          # uses "myapp:latest"
```

**Override at runtime** with `--args`:

```bash
xeq run build --args image=myapp:hotfix
```

**Fallback values** - use `|` to provide a default if a variable isn't set:

```toml
[build]
run = ["docker build -t {{@image | myapp:latest}} ."]
```

**Resolution order:** `--args` → local vars → global vars → fallback

> If a variable isn't defined and no fallback is set, xeq exits with an error. Pass `--allow-empty-vars` or add `allow_empty_vars` to options to skip this.

---

### 3. Arguments

Use positional placeholders `{{1}}`, `{{2}}` for values that change every run:

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

Mix named and positional args:

```bash
xeq run deploy --args env=production my-app
```

> Missing arguments cause an error. Use `--allow-empty-vars` to skip this.

---

### 4. Environment Variables

Reference environment variables with `{{$VARNAME}}`:

```toml
[deploy]
run = ["deploy --token {{$API_TOKEN}} --env {{$DEPLOY_ENV}}"]
```

xeq loads a `.env` file from the current directory automatically:

```bash
# .env
API_TOKEN=abc123
DEPLOY_ENV=production
```

Pass `--no-env` to skip loading `.env`.

> Missing env vars cause an error. Use `--allow-empty-vars` to skip this.

---

### 5. Nested Scripts

Call other scripts from within a script using the `xeq:` prefix:

```toml
[install]
run = ["npm install"]

[build]
run = ["npm run build"]

[deploy]
run = [
    "xeq:install",
    "xeq:build",
    "npm run deploy"
]
```

Running `xeq run deploy` runs `install` and `build` first, in order.

> xeq detects circular dependencies and exits. Add `allow_recursion` to options if you intentionally need a script to call itself.

---

### 6. Parallel Execution

Run all commands in a script at the same time:

```toml
[check]
parallel_threads = 4
run = [
    "cargo test",
    "cargo clippy",
    "cargo fmt --check"
]
```

Use `-p` to toggle parallel mode from the CLI, or `-p <threads>` to override the thread count:

```bash
xeq run check         # uses parallel_threads from TOML
xeq run check -p      # uses logical CPU count
xeq run check -p 8    # uses 8 threads
```

> Scripts with `cd` commands or `xeq:` calls cannot run in parallel. `xeq validate` catches this.

---

### 7. Global Configuration

Save a `xeq.toml` globally to run scripts from any directory:

```bash
xeq config ~/my-scripts/xeq.toml   # save once
xeq run git-cleanup --global        # run from anywhere
xeq list --global                   # list global scripts
```

---

### 8. Run Summary

Pass `--summary` to see every command and how long it took:

```bash
xeq run build --summary
```

```
command                        time   status
--------------------------------------------------
cargo test                     1.39s  succeeded
cargo clippy                   0.80s  succeeded
cargo fmt                      0.26s  succeeded
```

---

### 9. Events

Run additional commands when a script succeeds or fails:

```toml
[build]
run = ["cargo test", "cargo build"]
on_success = ["echo build passed"]
on_error = ["echo build failed"]
```

**Rules:**
- Events are ignored during parallel execution
- Events disable the run summary
- Events cannot be combined with `continue_on_err`

---

### 10. Custom Shells

Set a shell at the file level to run all commands with:

```toml
shell = "zsh"

[build]
run = ["cargo build", "cargo test"]
```

**Available shells:** `sh`, `bash`, `zsh`, `fish`, `cmd`, `powershell`

Defaults to `sh` on Linux/macOS and `cmd` on Windows.

---

## Examples

The [`examples/`](./examples) folder has ready-to-use TOML files for common workflows.

| File                  | Description                                 | Features Used                             |
| --------------------- | ------------------------------------------- | ----------------------------------------- |
| `react-tailwind.toml` | Scaffold and run a React + Tailwind project | variables, cd operators                   |
| `nextjs.toml`         | Next.js project setup and pipeline          | nested scripts, variables                 |
| `rust-project.toml`   | Rust checks, build and publish              | parallel, nested scripts                  |
| `docker.toml`         | Docker image and container management       | variables, nested scripts                 |
| `git-workflow.toml`   | Common git operations                       | variables, arguments                      |
| `nested-scripts.toml` | CI pipeline from reusable pieces            | nested scripts                            |
| `env-vars.toml`       | Deploy and notify using env vars            | `{{$VAR}}`, nested scripts                |
| `python-project.toml` | Virtualenv, checks and PyPI publish         | parallel, nested scripts, variables       |
| `database.toml`       | Migrations, seed, dump and restore          | env vars, arguments, nested scripts       |
| `monorepo.toml`       | Multi-package frontend workspace            | parallel, variables, nested scripts       |
| `aws-deploy.toml`     | ECR push and ECS deploy pipeline            | env vars, nested scripts                  |
| `go-project.toml`     | Go build, test and cross-compile            | parallel, nested scripts, variables       |
| `arguments.toml`      | Positional and named arg patterns           | arguments, variables                      |
| `script-options.toml` | Flag toggle mechanic demonstrations         | options, parallel, quiet, continue_on_err |

---

## How It Works

- xeq reads `xeq.toml` from the current directory, or a globally saved path with `--global`
- Commands run through `sh -c` on Linux/macOS and `cmd /C` on Windows by default
- `cd` commands update the working directory for all subsequent commands in that script
- Variables resolve in order: `--args` → local vars → global vars → fallback
- `.env` is loaded automatically before any script runs
- Script names are case-sensitive

---

## License

MIT - [LICENSE](LICENSE)