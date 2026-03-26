<div align="center">
<img src="./assets/logo_rounded.png" alt="logo" width="150"/>

# xeq

[![Crates.io](https://img.shields.io/crates/v/xeq)](https://crates.io/crates/xeq)
[![Downloads](https://img.shields.io/crates/d/xeq)](https://crates.io/crates/xeq)
[![License](https://img.shields.io/crates/l/xeq)](LICENSE)
[![Build](https://github.com/opmr0/xeq/actions/workflows/release.yml/badge.svg)](https://github.com/opmr0/xeq/actions)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

**xeq is a cross-platform CLI tool that runs sequences of commands from a single TOML file, making repetitive tasks fast and consistent.**

Every project has a setup ritual. Ten commands, always in the same order, run every time. Write them once in a `xeq.toml`, commit it, and anyone on any OS runs the exact same steps with one command.

</div>

---

## Table of Contents

- [Quick Start](#quick-start)
- [Comparison with other tools](#Comparison-with-other-tools)
- [Installation](#installation)
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
  - [9. Fallback script](#9-fallback-script)
  - [10. Custom shells](#10-custom-shells)
- [Examples](#examples)
- [How It Works](#how-it-works)
- [Contributing](#contributing)
- [License](#license)

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

xeq finds `xeq.toml` in the current directory automatically. No extra setup needed.

> Use `xeq init` to create a `xeq.toml` file, and xeq will generate one for you.

---

## Comparison with other tools

| Feature            | **xeq**              | **Makefile**       | **npm scripts** | **just**     |
| ------------------ | -------------------- | ------------------ | --------------- | ------------ |
| File type          | TOML                 | Makefile           | package.json    | Justfile     |
| Works on all OS    | Yes                  | Mostly Linux/macOS | Yes             | Yes          |
| Validation command | Yes (`xeq validate`) | No                 | No              | No           |
| Variables          | Yes                  | Yes                | Limited         | Yes          |
| Args support       | Yes                  | Limited            | Limited         | Yes          |
| Nested scripts     | Yes (`xeq://`)       | Yes                | No              | Yes          |
| Parallel run       | Yes                  | Yes (`-j`)         | No              | No           |
| .env support       | Yes (auto)           | No                 | No              | Yes (opt-in) |

---

## Commands

### `xeq init`

Creates a starter `xeq.toml` in the current directory. Will not overwrite an existing file.

```bash
xeq init [template]
```

You can chose a template to start with

##### Available templates

`android`, `ansible`, `astro`, `aws`, `bun`, `deno`, `django`, `docker`, `dotnet`, `elixir`, `expo`, `fastapi`, `flutter`, `git`, `go`,`hugo` ,`java-gradle`,`java-maven`, `kubernetes`, `laravel`, `monorepo`, `nestjs`, `nextjs`, `node`, `python`, `rails`,`react`, `rust`, `svelte`, `tauri`

---

### `xeq run <script> [flags]`

Runs a script by name. Commands execute one at a time in order. If any command fails, xeq stops, unless you pass `--continue-on-err`.

```bash
xeq run setup
xeq run build --continue-on-err
xeq run dev --quiet
xeq run test --parallel
xeq run create --args my-app
xeq run deploy --args env=prod
```

| Flag                       | Short | Description                                                                                                       |
| -------------------------- | ----- | ----------------------------------------------------------------------------------------------------------------- |
| `--continue-on-err`        | `-C`  | Keep going even if a command fails                                                                                |
| `--quiet`                  | `-q`  | Hide xeq's own log messages                                                                                       |
| `--clear`                  | `-c`  | Clear the terminal before each command                                                                            |
| `--parallel [threads_num]` | `-p`  | Run all commands at the same time with certain number of threads (default: your PC's logical processors(threads)) |
| `--args <values...>`       | `-a`  | Pass arguments into the script, positional or `key=value`                                                         |
| `--global`                 | `-g`  | Use the globally saved path instead of the local `xeq.toml`                                                       |
| `--summary`                | `-s`  | Print a summary table of commands and execution times after the script finishes                                   |
| `--allow-empty-args`       | `-A`  | Remove the restrictions on arguments and variables                                                                |
| `--no-env`                 |       | Skip loading the `.env` file                                                                                      |
| `--allow-recursion`        |       | Let a script call itself                                                                                          |

---

### `xeq list`

Shows all scripts in your TOML file, their names, descriptions, and commands.

```bash
xeq list
xeq list --global
```

---
### `xeq toml`

Shows how the TOML format should be

```
Fields for the whole file

shell   Optional - Set the shell to run the command with, windows default: cmd, Linux/MacOS default: sh
Supported shells: sh, zsh, fish, bash, cmd, powershell

[vars]          Set variables for the file level
var_name = "value"

Fields for the single script

run     Required - a group of commands to run

options  Optional - Set a group of options for the script
Available options: continue_on_err, allow_recursion, allow_empty_vars, clear, quite, summary

dir     Optional - Set the directory where the commands will run in

description     Optional - Set a description for the script

parallel_threads        Optional - Enable parallel execution and set a number of threads for the execution

fallback        Optional - Set a fallback for another script if the current script failed

vars.var_name    Set a variable for the script level
```

---

### `xeq validate`

Checks all scripts in your TOML file for errors without running anything.

```bash
xeq validate
xeq validate --global
```

Catches:

- Nested `xeq://` calls pointing to scripts that don't exist. see [Nested Scripts](#5-nested-scripts)
- `parallel` is enabled and there is a `cd` or `xeq://` in the commands. see [Parallel Execution](#6-parallel-execution)
- Undefined `{{@vars}}` not defined in vars. see [Variables](#2-variables)
- Circular dependencies between scripts
- `fallback` and `continue_on_err` set on the same script. see [Fallback script](#9-fallback-script)
- unsupported shells in the shell value. see [Custom Shells](#10-custom-shells)
- `dir` paths that don't exist
- parallel_thread filed equals 0 or 1

---

### `xeq config [path]`

Saves a TOML file path globally. See [Global Configuration](#7-global-configuration).

```bash
xeq config ~/my-scripts/xeq.toml   # save the path
xeq config                          # open the saved file in your default editor
```

### Commands aliases

xeq supports command aliases so `xeq run` is the same as `xeq r`

| command        | alias   |
| -------------- | ------- |
| `xeq run`      | `xeq r` |
| `xeq config`   | `xeq c` |
| `xeq init`     | `xeq i` |
| `xeq validate` | `xeq v` |

---

## TOML Format

A `xeq.toml` file contains named scripts. Each script needs at least a `run` array:

```toml
[my-script]
description = "What this script does"
parallel_threads = 6
dir = "./my_app"
options = ["quiet"]
run = [
    "command one",
    "command two"
]
```

- Script names are **case-sensitive**, `Build` and `build` are different scripts
- `run` **required**, contains all commands to run
- `description` _optional_, only shows in `xeq list`
- `parallel_threads` _optional_, enables parallel execution and set a number of threads for the execution. see [Parallel Execution](#6-parallel-execution)
- `dir` _optional_, is the path where the script will run, it can be an absolute or a relative path
- `options` _optional_. see [Script Options](#1-script-options)

---

## Features

### 1. Script Options

Bake default behavior into a script so you don't have to pass flags every time:

```toml
[build]
options = ["quiet"]
run = ["cargo build", "cargo test"]
```

Now `xeq run build` always runs quietly and in parallel, no flags needed.

**Available options:** `quiet`, `clear`, `continue_on_err`, `allow_recursion`, `summary`, `allow_empty_vars`

**Toggling:** CLI flags _toggle_ script options. If a script has `quiet` baked in and you pass `--quiet`, it turns quiet _off_ for that run.

```bash
xeq run build          # quiet ON  (from TOML)
xeq run build --quiet  # quiet OFF (toggled by CLI flag)
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

**Resolution order, most specific win:**

```
--args (runtime)  ->  local vars (per script)  ->  global vars (file-level)
```

> an error will happen if the vars isn't defined, you can prevent this by using `--allow-empty-vars` / `-A` flag or `allow_empty_flag` option and the raw `{{@varname}}` will be passed if no variables was passed

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

> an error will happen if the argument isn't passed, you can prevent this from happening using `--allow-empty-vars` / `-A` flag or `allow_empty_flag` option and the raw `{{n}}` will be passed if no arguments was provided

---

### 4. Environment Variables

Reference system environment variables in commands using `{{$VARNAME}}`:

```toml
[deploy]
run = ["deploy --token {{$API_TOKEN}} --env {{$DEPLOY_ENV}}"]
```

xeq automatically loads a `.env` file from the current directory if one exists:

```bash
# .env
API_TOKEN=abc123
DEPLOY_ENV=production
```

```bash
xeq run deploy   # API_TOKEN and DEPLOY_ENV loaded automatically
```

Pass `--no-env` to skip loading the `.env` file.

> an error will happen if the environment variable doesn't exists

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

> **Circular dependency protection:** xeq detects and exits on circular calls. Add `allow_recursion` to `options` or pass the `--allow-recursion` flag if you intentionally need this.

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

`parallel_threads` field enables parallel execution and set a number of threads for the execution

use `-p` to enable parallel execution and set the number of threads to your PC's threads or disable it if it's enable in the config file

use `-p <threads>` to enable parallel execution or override the parallel_threads value on a single run

```bash
xeq run check      # all three run at the same time
xeq run check -p 4   # same using CLI flag
```

> Scripts with `cd` commands or `xeq://` calls cannot run in parallel. Use `xeq validate` to catch this before running.

---

### 7. Global Configuration

xeq finds `xeq.toml` in your current directory automatically, no setup needed for project-level scripts.

For scripts you use across all your projects save a global file once and run it from anywhere:

```bash
xeq config ~/my-scripts/xeq.toml   # save once
xeq run git-cleanup --global        # run from any directory
xeq list --global                   # see all global scripts
```

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

## 9. Fallback script

Set a script to run automatically when any command in a script fails:

```toml
[build]
run = ["cargo build", "cargo test"]
fallback = "notify-failure"

[notify-failure]
run = ["echo build failed, notifying team"]
```

If any command in `build` fails, xeq immediately runs `notify-failure` and stops the rest of `build`.

The fallback script receives the same `--args` that were passed to the original script.

> **Note:** `fallback` and `continue_on_err` cannot be used together on the same script. xeq will exit with an error if both are set.

---

## 10. Custom shells

Set a shell to run your command with per file

```toml
shell = "zsh"

[build]
run = [
    "cargo build",
    "cargo test",
]
```

Available shells: `sh`,`bash`,`zsh`,`cmd`,`powershell`,`fish`

When the shell value isn't supported xeq will return an err

## Examples

The [`examples/`](./examples) folder has ready-to-use TOML files for common workflows.

| File                  | Description                                 | Key Features Used                         |
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

- xeq stores your TOML file path using the system config directory
- Commands run through `sh -c` on Linux/macOS and `cmd /C` on Windows
- `cd` commands update the working directory for all subsequent commands in that script
- Variables resolve in order: `--args` -> local vars -> global vars
- Environment variables are loaded from `.env` automatically before any script runs
- Script names are case-sensitive: `Build` and `build` are different scripts

---

## License

MIT - [LICENSE](LICENSE)
