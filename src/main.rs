use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

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

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

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
        script_name: Option<String>,

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

        #[arg(
            short = 'A',
            long,
            help = "Allow scripts to run even if some variables or arguments are missing"
        )]
        allow_empty_vars: bool,

        #[arg(short, long, num_args = 1.., value_name = "VALUES", help = "Pass arguments or variables to the script")]
        args: Option<Vec<String>>,
        #[arg(short, long, help = "Preview commands without executing them")]
        dry_run: bool,
        #[arg(short = 'e', long, help = "Disable events for this run")]
        no_events: bool,
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
    Init { template: Option<String> },

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
        #[arg(short, long, help = "Validate the sssssssssss")]
        runtime: bool,
    },
    #[clap(
        disable_help_flag = true,
        about = "Shows you how the xeq TOML format should be"
    )]
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
    let last_ctrl_c = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let last_ctrl_c_clone = last_ctrl_c.clone();

    ctrlc::set_handler(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last = last_ctrl_c_clone.load(Ordering::SeqCst);

        if now - last < 2000 {
            process::exit(1);
        } else {
            last_ctrl_c_clone.store(now, Ordering::SeqCst);
        }
    })
    .unwrap();

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
            allow_empty_vars,
            dry_run,
            no_events,
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
                allow_empty_vars,
                dry_run,
                no_events,
            )?;
        }
        Command::List { global } => cmd_list(global),
        Command::Init { template } => cmd_init(template)?,
        Command::Validate { global, runtime } => cmd_validate(global, runtime),
        Command::Toml => {
            let sep = "─".repeat(55).dimmed();

            println!("\n{}", "xeq toml format".bold().bright_green());
            println!("{}\n", sep);

            // File level
            println!("{}", "File".bold().bright_yellow());
            println!(
                "  {:<20} {}",
                "shell = \"bash\"".cyan(),
                "optional - sh, zsh, fish, bash, cmd, powershell".dimmed()
            );
            println!(
                "  {:<20} {}",
                "default = \"script\"".cyan(),
                "optional - script to run when no name is given".dimmed()
            );
            println!(
                "  {:<20} {}",
                "[vars]".bright_magenta(),
                "optional - file level variables".dimmed()
            );
            println!("  {:<20}", "  var_name = \"value\"".white());

            // Script fields
            println!("\n{}", "Script".bold().bright_yellow());
            println!("{}", "  [script-name]".bright_magenta());

            let fields: &[(&str, &str, &str)] = &[
                (
                    "run",
                    "= [\"cmd1\", \"cmd2\"]",
                    "required - commands to run in order",
                ),
                ("description", "= \"...\"", "optional - shown in xeq list"),
                ("dir", "= \"./path\"", "optional - working directory"),
                ("options", "= [\"...\"]", "optional - baked in flags"),
                (
                    "parallel_threads",
                    "= 4",
                    "optional - enables parallel execution",
                ),
                ("on_success", "= [\"cmd\"]", "optional - run on success"),
                ("on_error", "= [\"cmd\"]", "optional - run on error"),
                (
                    "vars.name",
                    "= \"value\"",
                    "optional - script level variable",
                ),
                (
                    "default",
                    "= \"script\"",
                    "optional - run when no script is given",
                ),
            ];

            for (key, val, desc) in fields {
                println!("  {:<18} {:<22} {}", key.cyan(), val.white(), desc.dimmed());
            }

            // Options
            println!("\n{}", "Options".bold().bright_yellow());
            let options: &[(&str, &str)] = &[
                ("quiet", "suppress xeq log messages"),
                ("clear", "clear terminal before each command"),
                ("continue_on_err", "keep running if a command fails"),
                ("allow_recursion", "allow a script to call itself"),
                ("allow_empty_vars", "skip errors for undefined variables"),
            ];
            for (opt, desc) in options {
                println!("  {:<20} {}", opt.cyan(), desc.dimmed());
            }

            // Variable types
            println!("\n{}", "Variables".bold().bright_yellow());
            let vars: &[(&str, &str)] = &[
                ("{{@var}}", "user defined variable"),
                ("{{@var | default}}", "user defined variable with fallback"),
                ("{{$ENV_VAR}}", "environment variable"),
                ("{{1}} {{2}}", "positional arguments"),
            ];
            for (syntax, desc) in vars {
                println!("  {:<26} {}", syntax.bright_blue(), desc.dimmed());
            }

            println!("\n{}", sep);
            println!(
                "{}",
                "run `xeq init` to create a starter xeq.toml"
                    .italic()
                    .bright_black()
            );
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
    script_name: Option<String>,
    continue_on_err: bool,
    clear: bool,
    quiet: bool,
    args: Option<Vec<String>>,
    parallel: Option<usize>,
    allow_recursion: bool,
    no_env: bool,
    global: bool,
    allow_empty_vars: bool,
    dry_run: bool,
    no_events: bool,
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
        allow_empty_vars,
        dry_run,
        no_events,
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| {
        err!("could not determine current directory, falling back to '.'");
        PathBuf::from(".")
    });

    if opts.dry_run {
        log!(false, "{}", "Running dry..".yellow())
    }

    let script_name = match script_name {
        Some(x) => x,
        None => match config.default.clone() {
            Some(x) => {
                log!(opts.quiet, "Running the default script '{x}'");
                x
            }
            None => {
                return Err(anyhow!("no script name provided and no default script set"));
            }
        },
    };

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

    let config = load_config_or_exit(global);
    let source = if global { "global config" } else { "xeq.toml" };

    println!(
        "\n{} scripts in {}:\n{}",
        "[xeq]".cyan().bold(),
        source,
        "─".repeat(40).dimmed()
    );

    // file level config
    if let Some(shell) = &config.shell {
        println!("  {} {}", "shell:".bright_black(), shell.white());
    }
    if let Some(default) = &config.default {
        println!("  {} {}", "default:".bright_black(), default.white());
    }
    if config.shell.is_some() || config.default.is_some() {
        println!();
    }

    for (name, script) in &config.scripts {
        let desc = script.description.as_deref().unwrap_or("no description");
        let dir = script.dir.as_deref().unwrap_or(".");
        let parallel = script
            .parallel_threads
            .map(|n| format!(" · {} threads", n))
            .unwrap_or_default();

        println!(
            "  {} {}{}",
            name.cyan().bold(),
            desc.dimmed().italic(),
            parallel.dimmed()
        );
        println!("  {} {}", "dir:".bright_black(), dir.white());
        for cmd in &script.run {
            println!("    {} {}", "›".dimmed(), cmd.yellow());
        }
        if let Some(on_success) = &script.on_success {
            println!(
                "  {} {}",
                "on_success:".bright_black(),
                on_success.join(", ").green()
            );
        }
        if let Some(on_error) = &script.on_error {
            println!(
                "  {} {}",
                "on_error:".bright_black(),
                on_error.join(", ").red()
            );
        }

        println!();
    }

    println!("{}", "─".repeat(40).dimmed());
    println!(
        "{}",
        format!("{} scripts", config.scripts.len())
            .dimmed()
            .italic()
    );
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

fn cmd_validate(global: bool, runtime: bool) {
    log!(
        false,
        "validating scripts in {}:",
        if global { "global config" } else { "xeq.toml" }
    );
    log!(false, "checking for parse errors");

    let config = load_config_or_exit(global);

    log!(false, "{}\n", "parsing passed".green());

    if validate(&config, runtime) {
        err!("some scripts failed validation");
        process::exit(1);
    } else {
        log!(false, "{}", "all scripts passed".green());
    }
}
