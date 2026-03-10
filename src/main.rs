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

use config::*;
use runner::*;

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
        #[arg(short, long, num_args = 1..)]
        args: Option<Vec<String>>,
    },
    List,
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
        } => {
            validate_or_exit();
            let mut visited = HashSet::new();
            let scripts = match read_scripts() {
                Ok(x) => x,
                Err(e) => {
                    err!("{}", e);
                    process::exit(1);
                }
            };

            let opts = RunOptions {
                continue_on_err,
                quiet,
                clear,
                parallel,
                allow_recursion,
            };

            run(script_name, &scripts, &mut visited, args, opts);
        }
        Command::List => {
            validate_or_exit();
            log!(false, "Listing tasks... \n");
            let content = read_scripts().unwrap();
            for s in content {
                println!(
                    "{} runs: --- options: {:?}",
                    s.0.cyan(),
                    s.1.options.unwrap_or_default()
                );
                for c in s.1.run.iter() {
                    println!("\t{}", c.yellow())
                }
            }
        }
    }
}
