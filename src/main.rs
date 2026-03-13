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

use config::*;
use runner::*;

use crate::validation::validate;

#[derive(Parser, Debug)]
#[clap(version, about, name = "xeq")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Config {
        path: Option<PathBuf>,
    },
    Run {
        script_name: String,
        #[arg(short = 'C', long, help = "Keep running even if a command fails")]
        continue_on_err: bool,
        #[arg(short, long, help = "Clear the screen between commands")]
        clear: bool,
        #[arg(short, long, help = "Suppress xeq output")]
        quiet: bool,
        #[arg(short, long)]
        parallel: bool,
        #[arg(long)]
        allow_recursion: bool,
        #[arg(long, short)]
        global: bool,
        #[arg(long)]
        no_env: bool,
        #[arg(short, long, num_args = 1.. ,value_name = "VALUES")]
        args: Option<Vec<String>>,
    },
    List {
        #[arg(long, short)]
        global: bool,
    },
    Init,
    Validate {
        #[arg(long, short)]
        global: bool,
    },
}

fn validate_or_exit() {
    if let Some(path) = load_path() {
        if validate_path(&path).is_err() {
            err!(
                "The commands TOML file has been deleted or moved.\n      Configure xeq using: xeq config <path/to/file.toml>"
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
            };

            let cwd = std::env::current_dir().unwrap_or_else(|_| {
                err!("Warning: could not determine current directory, falling back to '.'");
                PathBuf::from(".")
            });

            run(script_name, &config, &mut visited, args, opts, cwd);
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
                println!("")
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
                Ok(_) => log!(false, "created xeq.toml — run 'xeq run setup' to try it"),
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
