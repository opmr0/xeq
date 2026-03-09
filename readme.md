<br/>

<img src="./logo.png" alt="logo" width="120"/>

# xeq

Run sequences of commands from a JSON file with a single word.

Stop writing the same commands over and over. Define them once, run them anywhere.

```bash
xeq run setup
```

---

## Why xeq?

Setting up a new project usually means running 5-10 commands in order. You either remember them, keep them in a notes file, or write a shell script that only work from one place.

xeq fixes that. Write your commands in a JSON file once, commit it to your repo, and anyone on any OS can run the same setup with one command.

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

Create a `xeq.json` file in your project:

```json
{
    "setup": { // <-- the script name
        "run": [ // <-- the commands will run
            "npm create vite@latest my-app -- --template react",
            "cd my-app",
            "npm install",
            "npm install -D tailwindcss postcss autoprefixer"
        ]
    },
    "dev": {
        "run": [
            "cd my-app",
            "npm run dev"
        ]
    }
}
```

Tell xeq where your file is:
```bash
xeq config ./xeq.json
```

Run a script:
```bash
xeq run setup
```

---

## Commands

### `xeq config [path]`

Save the path to your JSON file. xeq will use it every time you run a script.

```bash
xeq config ./xeq.json
```

Running `xeq config` with no arguments opens the saved file in your default editor.

```bash
xeq config
```

---

### `xeq run <script>`

Run a named script from your JSON file. Commands execute sequentially — if one fails, xeq stops unless you pass `--continue-on-error`.

```bash
xeq run setup
xeq run build --continue-on-error
xeq run dev --quiet
```

| Flag | Short | Description |
| ---- | ----- | ----------- |
| `--continue-on-error` | `-C` | Keep running even if a command fails |
| `--quiet` | `-q` | Hide xeq's own output, only show command output |
| `--clear` | `-c` | Clear the screen before each command |

---

### `xeq list`

List all scripts in your JSON file and the commands they run.

```bash
xeq list
```

Output:
```
build runs:
    cargo fmt
    cargo clippy
    cargo build --release
```

---

## JSON Format

```json
{
    "script-name": {
        "run": [
            "command one",
            "command two",
            "command three"
        ]
    }
}
```

You can define as many scripts as you want in a single file. Each script is a named list of commands that run in order.

---

## Nested scripts 

You can define nested scripts using `xeq://task_name` syntax in the JSON file

#### example
```json
{
    "setup": {
        "run": [
            "npm install"
        ]
    },
    "build": {
        "run": [
            "npm run build"
        ]
    },
    "deploy": {
        "run": [
            "xeq://setup",
            "xeq://build",
            "npm run deploy"
        ]
    }
}
```

`deploy` script runs both `build` and `setup` scripts respectively without needing to run both

## Arguments

You can pass arguments to your scripts and reference them with `{{1}}`, `{{2}}`, etc.
```json
{
    "greet": {
        "run": [
            "echo Hello {{1}}!",
            "echo You are {{2}} years old."
        ]
    }
}
```
```
xeq run greet John 26
```

Output:
```
Hello Omar!
You are 25 years old.
```

Arguments are optional — if not provided, the placeholder will be passed as-is to the shell.


## How It Works

- Commands run sequentially, one at a time
- `cd` commands change the working directory for all commands that follow in the same script
- On failure, xeq exits with the same exit code as the failed command
- Works on Linux, macOS, and Windows with the same JSON file

---

## Examples

See the [`examples/`](./examples) folder for ready-to-use JSON files:

- `react-tailwind.json` — React + Tailwind CSS setup
- `nextjs.json` — Next.js project setup
- `rust-project.json` — Rust project workflow
- `docker-app.json` — Docker Compose workflow
- `git-workflow.json` — Common git operations
- `scripts-nesting.json` — Example on nested tasks

---



## License

MIT — [LICENSE](LICENSE)