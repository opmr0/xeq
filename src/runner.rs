use std::collections::HashMap;
use std::env;
use std::{collections::HashSet, path::PathBuf, process};

use colored::Colorize;

use crate::types::Config;
use crate::types::ScriptOption::*;

use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone, Copy)]
pub struct RunOptions {
    pub continue_on_err: bool,
    pub clear: bool,
    pub quiet: bool,
    pub parallel: Option<usize>,
    pub allow_recursion: bool,
    pub summary: bool,
    pub allow_empty_vars: bool,
}

struct CommandSummary {
    command: String,
    duration: f64,
    succeeded: bool,
}

pub fn replace_args(line: &str, args: &[String], allow_empty_vars: bool) -> Result<String, String> {
    let mut line = line.to_owned();
    let largest_placeholder: usize = line
        .split_whitespace()
        .filter(|x| !x.contains('@') || !x.contains('$'))
        .map(|x| {
            x.trim_end_matches("}}")
                .trim_start_matches("{{")
                .parse()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    if !args.is_empty() {
        if largest_placeholder > args.len() {
            return Err(format!("not enough arguments `{}`", line));
        }
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{{{}}}}}", i + 1);
            line = line.replace(&placeholder, arg);
        }
    } else if !allow_empty_vars && largest_placeholder != 0 {
        return Err(format!(
            "arguments for the placeholders are required `{}`",
            line
        ));
    }

    Ok(line)
}

pub fn replace_vars(
    line: &str,
    global_vars: &Option<HashMap<String, String>>,
    local_vars: &Option<HashMap<String, String>>,
    args: &HashMap<String, String>,
    allow_empty_vars: bool,
) -> Result<String, String> {
    let mut line = line.to_owned();

    let mut i = 0;
    while let Some(start) = line[i..].find("{{@") {
        let start = i + start;
        let end = match line[start..].find("}}") {
            Some(e) => start + e + 2,
            None => break,
        };

        let key = &line[start + 3..end - 2].to_owned();

        let value = match args
            .get(key)
            .or_else(|| local_vars.as_ref()?.get(key))
            .or_else(|| global_vars.as_ref()?.get(key))
        {
            Some(x) => x,
            None if allow_empty_vars => &format!("{{{{@{}}}}}", key.clone()),
            None => {
                err!("undefined variable '{{{{@{}}}}}' is not set", key);
                process::exit(1)
            }
        };

        line.replace_range(start..end, value);
        i = start + value.len();
    }

    Ok(line)
}

pub fn replace_env(line: &str) -> Result<String, String> {
    let mut line = line.to_owned();

    let mut i = 0;
    while let Some(start) = line[i..].find("{{$") {
        let start = i + start;
        let end = match line[start..].find("}}") {
            Some(e) => start + e + 2,
            None => break,
        };

        let key = &line[start + 3..end - 2];

        let value = match env::var(key) {
            Ok(x) => x,
            Err(_) => {
                err!("environment variable '{{{{${}}}}}' is not set", key);
                process::exit(1)
            }
        };

        line.replace_range(start..end, &value);
        i = start + value.len();
    }

    Ok(line)
}

pub fn parse_args(args: &[String]) -> (HashMap<String, String>, Vec<String>) {
    let mut named = HashMap::new();
    let mut positional = Vec::new();

    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            named.insert(key.to_string(), value.to_string());
        } else {
            positional.push(arg.clone());
        }
    }

    (named, positional)
}

pub fn spawn_command(line: &str, cwd: &PathBuf) -> std::process::Child {
    #[cfg(target_os = "windows")]
    return std::process::Command::new("cmd")
        .args(["/C", line])
        .current_dir(cwd)
        .spawn()
        .expect("failed to spawn process - is 'cmd.exe' available?");

    #[cfg(not(target_os = "windows"))]
    return std::process::Command::new("sh")
        .args(["-c", line])
        .current_dir(cwd)
        .spawn()
        .expect("failed to spawn process - is 'sh' available?");
}

pub fn run(
    script_name: String,
    config: &Config,
    visited: &mut HashSet<String>,
    args: Option<Vec<String>>,
    mut opts: RunOptions,
    mut cwd: PathBuf,
) -> Result<(), String> {
    let scripts = &config.scripts;

    let script = match scripts.get(&script_name) {
        Some(x) => x,
        None => {
            err!(
                "script '{}' not found , run 'xeq list' to see available scripts",
                script_name
            );
            if !opts.continue_on_err {
                process::exit(1);
            } else {
                return Ok(());
            }
        }
    };
    let mut summary: Vec<CommandSummary> = Vec::new();

    if let Some(script_options) = &script.options {
        if script_options.contains(&ContinueOnErr) {
            opts.continue_on_err = !opts.continue_on_err;
        }
        if script_options.contains(&Quiet) {
            opts.quiet = !opts.quiet
        }
        if script_options.contains(&Clear) {
            opts.clear = !opts.clear
        }
        if script_options.contains(&AllowRecursion) {
            opts.allow_recursion = !opts.allow_recursion
        }
        if script_options.contains(&Summary) {
            opts.summary = !opts.summary
        }
        if script_options.contains(&AllowEmptyVars) {
            opts.allow_empty_vars = !opts.allow_empty_vars
        }
    }

    if opts.continue_on_err && script.fallback.is_some() {
        return Err("fallback and continue_on_err cannot be used together".to_string());
    }

    let parallel = match opts.parallel {
        Some(x) => Some(x),
        None if script.parallel_threads.is_some()
            && std::env::args().any(|x| x == "-p" || x == "--parallel") =>
        {
            None
        }
        None if script.parallel_threads.is_some() => script.parallel_threads,
        None if std::env::args().any(|x| x == "-p" || x == "--parallel") => Some(num_cpus::get()),
        None => None,
    };

    if let Some(dir) = &script.dir {
        let new_path = cwd.join(dir);
        match new_path.canonicalize() {
            Ok(resolved) => cwd = resolved,
            Err(e) => {
                return Err(format!("dir '{}': {}", dir, e));
            }
        }
    }

    let (named_args, positional_args) = parse_args(args.as_deref().unwrap_or(&[]));
    for (i, line) in script.run.iter().enumerate() {
        if opts.clear {
            clearscreen::clear().unwrap();
        }

        let line = replace_vars(
            line,
            &config.vars,
            &script.vars,
            &named_args,
            opts.allow_empty_vars,
        )?;
        let line = replace_args(&line, &positional_args, opts.allow_empty_vars)?;
        let line = replace_env(&line)?;

        if parallel.is_some() {
            let has_cd = script.run.iter().any(|l| l.starts_with("cd "));
            let has_nested = script.run.iter().any(|l| l.starts_with("xeq://"));

            if has_cd || has_nested {
                return Err(format!(
                    "Script '{}' contains {} , cannot run in parallel mode. \
            Remove the parallel option or restructure the script.",
                    script_name,
                    match (has_cd, has_nested) {
                        (true, true) => "'cd' and nested 'xeq://' calls",
                        (true, false) => "'cd' commands",
                        _ => "nested 'xeq://' calls",
                    }
                ));
            }
        }

        if let Some(n) = parallel {
            if n <= 1 {
                return Err("parallel_threads must be greater than 1".to_owned());
            }
            log!(opts.quiet, "{}", "running commands in parallel".purple());

            let resolved_lines: Vec<String> = script
                .run
                .iter()
                .map(|line| {
                    let line = replace_vars(
                        line,
                        &config.vars,
                        &script.vars,
                        &named_args,
                        opts.allow_empty_vars,
                    )
                    .unwrap_or_else(|e| {
                        err!("{}", e);
                        process::exit(1);
                    });
                    let line = replace_args(&line, &positional_args, opts.allow_empty_vars)
                        .unwrap_or_else(|e| {
                            err!("{}", e);
                            process::exit(1);
                        });
                    replace_env(&line).unwrap_or_else(|e| {
                        err!("{}", e);
                        process::exit(1);
                    })
                })
                .collect();

            let cwd = cwd.clone();
            let pool = threadpool::ThreadPool::new(n);
            let (tx, rx): (Sender<bool>, Receiver<bool>) = channel();

            for line in resolved_lines {
                let cwd = cwd.clone();
                let tx = tx.clone();
                pool.execute(move || {
                    let status = spawn_command(&line, &cwd)
                        .wait()
                        .expect("failed to wait for child process");

                    let success = status.success();
                    tx.send(success).expect("Failed to send status");
                });
            }

            drop(tx);

            let mut all_succeeded = true;
            for success in rx.iter() {
                if !success {
                    all_succeeded = false;
                    if !opts.continue_on_err {
                        break;
                    }
                }
            }

            pool.join();

            if all_succeeded {
                return Ok(());
            } else {
                Err("One or more commands failed.".to_owned())?;
            }
        }

        let mut cwd = cwd.clone();
        let total = script.run.len();

        log!(opts.quiet, "[{}/{}] {}", i + 1, total, line.yellow());

        if let Some(name) = line.strip_prefix("xeq://") {
            let name = name.to_owned();
            if visited.contains(&name) && !opts.allow_recursion {
                err!(
                    "circular dependency detected: '{}' calls '{}' which is already running",
                    script_name,
                    name
                );
                process::exit(1);
            }
            visited.insert(name.clone());
            log!(opts.quiet, "running nested script '{}'", name.purple());
            run(name.clone(), config, visited, args.clone(), opts, cwd)?;
            visited.remove(&name);
            continue;
        }

        if let Some(arg) = line.strip_prefix("cd ") {
            let arg = arg.trim();

            let (dir, rest, separator) = if let Some((d, r)) = arg.split_once("&&") {
                (d.trim(), Some(r.trim()), Some("&&"))
            } else if let Some((d, r)) = arg.split_once("||") {
                (d.trim(), Some(r.trim()), Some("||"))
            } else if let Some((d, r)) = arg.split_once(';') {
                (d.trim(), Some(r.trim()), Some(";"))
            } else if let Some((d, r)) = arg.split_once('&') {
                (d.trim(), Some(r.trim()), Some("&"))
            } else {
                (arg, None, None)
            };

            let (dir, negate) = if dir.starts_with('!') {
                (dir.trim_start_matches('!').trim(), true)
            } else {
                (dir, false)
            };

            let new_path = if dir.is_empty() {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
            } else {
                cwd.join(dir)
            };

            let cd_result = new_path.canonicalize();

            let cd_succeeded = match &cd_result {
                Ok(_) => !negate,
                Err(_) => negate,
            };

            match cd_result {
                Ok(resolved) => {
                    if !negate {
                        log!(
                            opts.quiet,
                            "Changing directory to {}",
                            resolved.display().to_string().yellow()
                        );
                        cwd = resolved;
                    } else {
                        err!(
                            "cd '{}': directory exists but negation (!) requires it not to",
                            dir
                        );
                        if !opts.continue_on_err {
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    if !negate {
                        err!("cd '{}': {}", new_path.display(), e);
                    }
                }
            }

            if let Some(rest) = rest {
                if !rest.is_empty() {
                    match separator {
                        Some("&&") => {
                            if cd_succeeded {
                                let status = spawn_command(rest, &cwd)
                                    .wait()
                                    .expect("failed to wait for child process");
                                if !status.success() {
                                    err!(
                                        "'{}' exited with code {}",
                                        rest,
                                        status.code().unwrap_or(-1)
                                    );
                                    if !opts.continue_on_err {
                                        process::exit(status.code().unwrap_or(1));
                                    }
                                } else {
                                    log!(opts.quiet, "{}", "done".green());
                                }
                            }
                        }
                        Some("||") => {
                            if !cd_succeeded {
                                let status = spawn_command(rest, &cwd)
                                    .wait()
                                    .expect("failed to wait for child process");
                                if !status.success() {
                                    err!(
                                        "'{}' exited with code {}",
                                        rest,
                                        status.code().unwrap_or(-1)
                                    );
                                    if !opts.continue_on_err {
                                        process::exit(status.code().unwrap_or(1));
                                    }
                                } else {
                                    log!(opts.quiet, "{}", "done".green());
                                }
                            }
                        }
                        Some(";") => {
                            let status = spawn_command(rest, &cwd)
                                .wait()
                                .expect("failed to wait for child process");
                            if !status.success() {
                                err!(
                                    "'{}' exited with code {}",
                                    rest,
                                    status.code().unwrap_or(-1)
                                );
                                if !opts.continue_on_err {
                                    process::exit(status.code().unwrap_or(1));
                                }
                            } else {
                                log!(opts.quiet, "{}", "done".green());
                            }
                        }
                        Some("&") => {
                            #[allow(clippy::zombie_processes)]
                            let _ = spawn_command(rest, &cwd);
                            log!(
                                opts.quiet,
                                "'{}' {}",
                                rest,
                                "spawned in background".purple()
                            );
                        }
                        _ => {}
                    }
                }
            }

            continue;
        }

        let start = std::time::Instant::now();
        let status = spawn_command(&line, &cwd)
            .wait()
            .expect("failed to wait for child process");
        let duration = start.elapsed().as_secs_f64();

        if !status.success() {
            err!(
                "'{}' exited with code {}",
                line,
                status.code().unwrap_or(-1)
            );
            if let Some(s) = &script.fallback {
                log!(opts.quiet, "{} '{s}'", "falling back to".purple());
                run(s.clone(), config, visited, args.clone(), opts, cwd)?;
                break;
            } else if !opts.continue_on_err {
                process::exit(status.code().unwrap_or(1));
            }
        } else {
            log!(opts.quiet, "{} in {:.2}s", "done".green(), duration);
        }
        if opts.summary {
            summary.push(CommandSummary {
                command: line,
                duration,
                succeeded: status.success(),
            });
        }
    }

    log!(
        opts.quiet,
        "script '{}' {}",
        script_name,
        "completed".green().bold()
    );
    if opts.summary {
        println!("\n {:<30} time   status", "command");
        println!("{}", "-".repeat(50));
        for CommandSummary {
            command,
            duration,
            succeeded,
        } in &summary
        {
            println!(
                "{:<30} {:.2}s {}",
                if command.len() > 26 {
                    format!("{}...", &command[..26])
                } else {
                    command.clone()
                },
                duration,
                if *succeeded {
                    "succeeded".green()
                } else {
                    "failed".red()
                }
            );
        }
        println!();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_error_when_not_enough_values() {
        let line = "echo {{1}} {{2}}";
        let args = vec!["only_one".to_string()];
        let result = replace_args(line, &args, false);
        assert!(result.is_err());
    }

    #[test]
    fn args_allow_empty_vars_keeps_placeholders() {
        let line = "echo {{1}} {{2}}";
        let result = replace_args(line, &[], true).unwrap();
        assert_eq!(result, "echo {{1}} {{2}}");
    }

    #[test]
    fn replace_env_multiple_vars() {
        std::env::set_var("XEQ_A", "A");
        std::env::set_var("XEQ_B", "B");
        let result = replace_env("echo {{$XEQ_A}} {{$XEQ_B}}").unwrap();
        assert_eq!(result, "echo A B");
        std::env::remove_var("XEQ_A");
        std::env::remove_var("XEQ_B");
    }

    #[test]
    fn parse_args_mixed_named_and_positional() {
        let args = vec![
            "key=value".to_string(),
            "pos1".to_string(),
            "another=123".to_string(),
            "pos2".to_string(),
        ];
        let (named, positional) = parse_args(&args);
        assert_eq!(named.get("key").unwrap(), "value");
        assert_eq!(named.get("another").unwrap(), "123");
        assert_eq!(positional, vec!["pos1".to_string(), "pos2".to_string()]);
    }

    #[test]
    fn replace_vars_with_allow_empty_keeps_unknown() {
        let result = replace_vars(
            "echo {{@missing}}",
            &None,
            &None,
            &std::collections::HashMap::new(),
            true,
        )
        .unwrap();
        assert_eq!(result, "echo {{@missing}}");
    }

    #[test]
    fn replace_vars_nested_multiple_sources() {
        let mut global = std::collections::HashMap::new();
        global.insert("a".to_string(), "1".to_string());

        let mut local = std::collections::HashMap::new();
        local.insert("b".to_string(), "2".to_string());

        let mut args = std::collections::HashMap::new();
        args.insert("c".to_string(), "3".to_string());

        let result = replace_vars(
            "echo {{@a}} {{@b}} {{@c}}",
            &Some(global),
            &Some(local),
            &args,
            false,
        )
        .unwrap();

        assert_eq!(result, "echo 1 2 3");
    }

    #[test]
    fn replace_env_no_placeholders() {
        let result = replace_env("echo hello").unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn parse_args_empty_input() {
        let args: Vec<String> = vec![];
        let (named, positional) = parse_args(&args);
        assert!(named.is_empty());
        assert!(positional.is_empty());
    }

    #[test]
    fn replace_args_large_index() {
        let line = "echo {{3}}";
        let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = replace_args(line, &args, false).unwrap();
        assert_eq!(result, "echo c");
    }

    #[test]
    fn replace_vars_partial_known_partial_unknown_allow() {
        let mut global = std::collections::HashMap::new();
        global.insert("a".to_string(), "1".to_string());

        let result = replace_vars(
            "echo {{@a}} {{@b}}",
            &Some(global),
            &None,
            &std::collections::HashMap::new(),
            true,
        )
        .unwrap();

        assert_eq!(result, "echo 1 {{@b}}");
    }
}
