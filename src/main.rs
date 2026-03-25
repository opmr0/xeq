use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process;

#[macro_use]
mod macros;
mod config;
mod runner;
mod types;
mod validation;

use config::{load_path, read_scripts, save_path, validate_path};
use runner::{run, RunOptions};

use crate::validation::validate;

#[derive(Parser, Debug)]
#[clap(
    version,
    about = "xeq is a cross-platform CLI tool that runs sequences of commands from a single TOML file, making repetitive tasks fast and consistent.",
    name = "xeq"
)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(alias = "c", about = "Set or open the path to your xeq configuration")]
    Config {
        #[arg(
            value_name = "PATH",
            help = "Path to a custom xeq.toml file (optional)"
        )]
        path: Option<PathBuf>,
    },

    #[command(alias = "r", about = "Run a named script from your xeq.toml")]
    Run {
        #[arg(value_name = "SCRIPT_NAME", help = "The name of the script to execute")]
        script_name: String,

        #[arg(
            short = 'C',
            long,
            help = "Continue running remaining commands even if one fails"
        )]
        continue_on_err: bool,

        #[arg(short, long, help = "Clear the terminal before executing each command")]
        clear: bool,

        #[arg(short, long, help = "Suppress xeq output and only show command output")]
        quiet: bool,

        #[arg(
            short,
            long,
            value_name = "THREADS",
            num_args = 0..=1,
            help = "Run all commands in the script in parallel"
        )]
        parallel: Option<usize>,

        #[arg(
            long,
            help = "Allow a script to call itself (for intentional recursion)"
        )]
        allow_recursion: bool,

        #[arg(
            long,
            short,
            help = "Use the global configuration file instead of the local one"
        )]
        global: bool,

        #[arg(long, help = "Skip loading environment variables from a .env file")]
        no_env: bool,

        #[arg(
            long,
            short,
            help = "Display a summary of commands and execution times after the script finishes"
        )]
        summary: bool,

        #[arg(
            long,
            short = 'A',
            help = "Allow scripts to run even if some arguments or variables are missing"
        )]
        allow_empty_vars: bool,

        #[arg(short, long, num_args = 1.., value_name = "VALUES", help = "Pass arguments or variables to the script at runtime")]
        args: Option<Vec<String>>,
    },

    #[command(
        alias = "l",
        about = "List all available scripts in the current or global xeq.toml"
    )]
    List {
        #[arg(
            long,
            short,
            help = "Show scripts from the global configuration instead of the local one"
        )]
        global: bool,
    },

    #[command(
        alias = "i",
        about = "Create a starter xeq.toml in the current directory"
    )]
    Init,

    #[command(alias = "v", about = "Validate your scripts without running them")]
    Validate {
        #[arg(
            long,
            short,
            help = "Validate scripts in the global configuration instead of the local one"
        )]
        global: bool,
    },
}

fn validate_or_exit() {
    if let Some(path) = load_path() {
        if validate_path(&path).is_err() {
            err!(
                "The commands TOML file has been deleted or moved.\nConfigure xeq using: xeq config <path/to/file.toml>"
            );
            process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Config { path } => {
            if let Some(path) = path {
                if let Err(e) = save_path(path) {
                    err!("{}", e);
                } else {
                    println!(
                        "{} {}",
                        "[xeq]".cyan().bold(),
                        "Configuration saved successfully!".green()
                    );
                }
            } else {
                validate_or_exit();
                let path = match load_path() {
                    Some(x) => x,
                    None => {
                        err!(
                            "xeq is not configured.\n      Configure it using: xeq config <path/to/file.toml>"
                        );
                        process::exit(1);
                    }
                };
                if let Err(e) = open::that(&path) {
                    err!("Failed to open {}: {}", path.display(), e);
                }
            }
        }

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
            if global {
                validate_or_exit()
            };
            let mut visited = HashSet::new();
            let config = match read_scripts(global) {
                Ok(x) => x,
                Err(e) => {
                    err!("{}", e);
                    process::exit(1);
                }
            };

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
                err!("Warning: could not determine current directory, falling back to '.'");
                PathBuf::from(".")
            });

            match run(script_name, &config, &mut visited, args, opts, cwd) {
                Ok(_) => {}
                Err(e) => {
                    err!("{}", e);
                    process::exit(1);
                }
            };
        }
        Command::List { global } => {
            if global {
                validate_or_exit()
            };
            log!(
                false,
                "scripts in {}:",
                if global { "global config" } else { "xeq.toml" }
            );
            let content = match read_scripts(global) {
                Ok(x) => x.scripts,
                Err(e) => {
                    err!("{}", e);
                    process::exit(1);
                }
            };
            for s in content {
                println!(
                    "{} --- {} \ndir: {} \nruns:",
                    s.0.cyan().bold(),
                    s.1.description
                        .unwrap_or("No description provided".to_owned())
                        .italic(),
                    s.1.dir.unwrap_or("None".to_owned()),
                );
                for c in s.1.run.iter() {
                    println!("\t{}", c.yellow())
                }
                println!()
            }
        }
        Command::Init => {
            let path = "./xeq.toml";

            if std::path::Path::new(path).exists() {
                err!("xeq.toml already exists in this directory");
                return;
            }

            let content = r#"[setup]
run = [
    "echo hello from xeq"
]
"#;

            match std::fs::write(path, content) {
                Ok(_) => log!(false, "created xeq.toml, run 'xeq run setup' to try it"),
                Err(e) => err!("could not create xeq.toml: {}", e),
            }
        }
        Command::Validate { global } => {
            log!(
                false,
                "validating scripts in {}:",
                if global { "global config" } else { "xeq.toml" }
            );
            log!(false, "checking parse errors");
            let config = match read_scripts(global) {
                Ok(x) => x,
                Err(e) => {
                    err!("{}", e);
                    process::exit(1)
                }
            };
            log!(false, "{} \n", "parsing passed".green());
            if validate(&config) {
                err!("some scripts failed")
            } else {
                log!(false, "{}", "all scripts passed".green())
            };
        }
    }
}
