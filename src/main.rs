use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

#[macro_use]
mod macros;
mod config;
mod runner;
mod templates;
mod types;
mod validation;

use config::{load_path, read_scripts, save_path, validate_path};
use runner::{run, RunOptions};
use templates::get_template;
use validation::validate;

#[derive(Parser, Debug)]
#[clap(
    version,
    name = "xeq",
    about = "xeq runs sequences of commands from a single TOML file, making repetitive tasks fast and consistent."
)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(alias = "c", about = "Set or open the path to your xeq config file")]
    Config {
        #[arg(value_name = "PATH", help = "Path to a xeq.toml file (optional)")]
        path: Option<PathBuf>,
    },

    #[command(alias = "r", about = "Run a named script from your xeq.toml")]
    Run {
        #[arg(value_name = "SCRIPT_NAME", help = "Name of the script to run")]
        script_name: String,

        #[arg(short = 'C', long, help = "Keep running even if a command fails")]
        continue_on_err: bool,

        #[arg(short, long, help = "Clear the terminal before each command")]
        clear: bool,

        #[arg(short, long, help = "Hide xeq log lines, only show command output")]
        quiet: bool,

        #[arg(
            short,
            long,
            value_name = "THREADS",
            num_args = 0..=1,
            help = "Run all commands in parallel (optionally set thread count)"
        )]
        parallel: Option<usize>,

        #[arg(long, help = "Allow a script to call itself")]
        allow_recursion: bool,

        #[arg(
            short,
            long,
            help = "Use the globally saved xeq.toml instead of the local one"
        )]
        global: bool,

        #[arg(long, help = "Skip loading .env from the current directory")]
        no_env: bool,

        #[arg(short, long, help = "Print a timing summary after the script finishes")]
        summary: bool,

        #[arg(
            short = 'A',
            long,
            help = "Allow scripts to run even if some variables or arguments are missing"
        )]
        allow_empty_vars: bool,

        #[arg(short, long, num_args = 1.., value_name = "VALUES", help = "Pass arguments or variables to the script")]
        args: Option<Vec<String>>,
    },

    #[command(
        alias = "l",
        about = "List all scripts in the current or global xeq.toml"
    )]
    List {
        #[arg(
            short,
            long,
            help = "List from the global config instead of the local one"
        )]
        global: bool,
    },

    #[command(
        alias = "i",
        about = "Create a starter xeq.toml in the current directory"
    )]
    Init {
        template: Option<String>,
    },

    #[command(
        alias = "v",
        about = "Check your scripts for errors without running them"
    )]
    Validate {
        #[arg(
            short,
            long,
            help = "Validate the global config instead of the local one"
        )]
        global: bool,
    },
    Toml,
}

fn validate_or_exit() {
    if let Some(path) = load_path() {
        if validate_path(&path).is_err() {
            err!(
                "The config file has been deleted or moved.\nConfigure xeq using: xeq config <path/to/file.toml>"
            );
            process::exit(1);
        }
    }
}

fn load_config_or_exit(global: bool) -> types::Config {
    match read_scripts(global) {
        Ok(c) => c,
        Err(e) => {
            err!("{}", e);
            process::exit(1);
        }
    }
}

fn main() {
    if let Err(e) = run_cli() {
        err!("{:?}", e);
        process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Config { path } => cmd_config(path)?,
        Command::Run {
            script_name,
            continue_on_err,
            clear,
            quiet,
            args,
            parallel,
            allow_recursion,
            no_env,
            global,
            summary,
            allow_empty_vars,
        } => {
            cmd_run(
                script_name,
                continue_on_err,
                clear,
                quiet,
                args,
                parallel,
                allow_recursion,
                no_env,
                global,
                summary,
                allow_empty_vars,
            )?;
        }
        Command::List { global } => cmd_list(global),
        Command::Init { template } => cmd_init(template)?,
        Command::Validate { global } => cmd_validate(global),
        Command::Toml => {
            // ! TEMP
            println!("Fields for the whole file\n");
            println!("shell\tOptional - Set the shell to run the command with, windows default: cmd, Linux/MacOS default: sh");
            println!("Supported shells: sh, zsh, fish, bash, cmd, powershell\n");
            println!("[vars]\t\tSet variables for the file level\nvar_name = \"value\"\n");
            println!("Fields for the single script\n");
            println!("run\tRequired - a group of commands to run\n");
            println!("options\t Optional - Set a group of options for the script");
            println!("Available options: continue_on_err, allow_recursion, allow_empty_vars, clear, quite, summary\n");
            println!("dir\tOptional - Set the directory where the commands will run in\n");
            println!("description\tOptional - Set a description for the script\n");
            println!("parallel_threads\tOptional - Enable parallel execution and set a number of threads for the execution\n");
            println!("fallback\tOptional - Set a fallback for another script if the current script failed\n");
            println!("vars.var_name\t Set a variable for the script level")
        }
    }

    Ok(())
}

fn cmd_config(path: Option<PathBuf>) -> Result<()> {
    if let Some(path) = path {
        save_path(path).context("failed to save config")?;
        println!(
            "{} {}",
            "[xeq]".cyan().bold(),
            "configuration saved.".green()
        );
    } else {
        validate_or_exit();
        let path = load_path().ok_or_else(|| {
            anyhow::anyhow!("xeq is not configured.\n      Run: xeq config <path/to/file.toml>")
        })?;
        open::that(&path).with_context(|| format!("failed to open {}", path.display()))?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_run(
    script_name: String,
    continue_on_err: bool,
    clear: bool,
    quiet: bool,
    args: Option<Vec<String>>,
    parallel: Option<usize>,
    allow_recursion: bool,
    no_env: bool,
    global: bool,
    summary: bool,
    allow_empty_vars: bool,
) -> Result<()> {
    if global {
        validate_or_exit();
    }

    let config = load_config_or_exit(global);

    if !no_env {
        dotenvy::dotenv().ok();
    }

    let opts = RunOptions {
        continue_on_err,
        quiet,
        clear,
        parallel,
        allow_recursion,
        summary,
        allow_empty_vars,
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| {
        err!("could not determine current directory, falling back to '.'");
        PathBuf::from(".")
    });

    let mut visited = HashSet::new();
    run(script_name, &config, &mut visited, args, opts, cwd).unwrap_or_else(|e| {
        err!("{}", e);
        process::exit(1);
    });

    Ok(())
}

fn cmd_list(global: bool) {
    if global {
        validate_or_exit();
    }

    log!(
        false,
        "scripts in {}:",
        if global { "global config" } else { "xeq.toml" }
    );

    let config = load_config_or_exit(global);

    for (name, script) in &config.scripts {
        println!(
            "{} --- {}\ndir: {}\nruns:",
            name.cyan().bold(),
            script
                .description
                .as_deref()
                .unwrap_or("no description")
                .italic(),
            script.dir.as_deref().unwrap_or("none"),
        );
        for cmd in &script.run {
            println!("\t{}", cmd.yellow());
        }
        println!();
    }
}

fn cmd_init(template: Option<String>) -> Result<()> {
    if Path::new("xeq.toml").exists() {
        err!("xeq.toml already exists");
        return Ok(());
    }

    let content = get_template(template);
    let mut file = File::create("xeq.toml").context("failed to create xeq.toml")?;
    file.write_all(content)
        .context("failed to write xeq.toml")?;
    log!(false, "xeq.toml created, edit it then run xeq run <script>");
    Ok(())
}

fn cmd_validate(global: bool) {
    log!(
        false,
        "validating scripts in {}:",
        if global { "global config" } else { "xeq.toml" }
    );
    log!(false, "checking for parse errors");

    let config = load_config_or_exit(global);

    log!(false, "{}\n", "parsing passed".green());

    if validate(&config) {
        err!("some scripts failed validation");
        process::exit(1);
    } else {
        log!(false, "{}", "all scripts passed".green());
    }
}
