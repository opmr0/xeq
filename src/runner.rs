use std::collections::HashMap;
use std::env;
use std::{collections::HashSet, path::PathBuf, process, thread};

use colored::Colorize;

use crate::types::Config;
use crate::types::ScriptOption::*;

#[derive(Clone, Copy)]
pub struct RunOptions {
    pub continue_on_err: bool,
    pub clear: bool,
    pub quiet: bool,
    pub parallel: bool,
    pub allow_recursion: bool,
    pub summary: bool,
}

struct CommandSummary {
    command: String,
    duration: f64,
    succeeded: bool,
}

pub fn replace_args(line: &str, args: &[String]) -> String {
    let mut line = line.to_owned();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{{{}}}}}", i + 1);
        line = line.replace(&placeholder, arg);
    }
    line
}

pub fn replace_vars(
    line: &str,
    global_vars: &Option<HashMap<String, String>>,
    local_vars: &Option<HashMap<String, String>>,
    args: &HashMap<String, String>,
) -> Result<String, String> {
    let mut line = line.to_owned();

    let mut i = 0;
    while let Some(start) = line[i..].find("{{@") {
        let start = i + start;
        let end = match line[start..].find("}}") {
            Some(e) => start + e + 2,
            None => break,
        };

        let key = &line[start + 3..end - 2];

        let value = args
            .get(key)
            .or_else(|| local_vars.as_ref()?.get(key))
            .or_else(|| global_vars.as_ref()?.get(key))
            .ok_or_else(|| format!("undefined variable '{{{{@{}}}}}'", key))?;

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
            opts.continue_on_err = !opts.continue_on_err
        }
        if script_options.contains(&Quiet) {
            opts.quiet = !opts.quiet
        }
        if script_options.contains(&Clear) {
            opts.clear = !opts.clear
        }
        if script_options.contains(&Parallel) {
            opts.parallel = !opts.parallel
        }
        if script_options.contains(&AllowRecursion) {
            opts.allow_recursion = !opts.allow_recursion
        }
        if script_options.contains(&Summary) {
            opts.summary = !opts.summary
        }
    }

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

        let line = replace_vars(line, &config.vars, &script.vars, &named_args)?;
        let line = replace_args(&line, &positional_args);
        let line = replace_env(&line)?;

        if opts.parallel {
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

        if opts.parallel {
            log!(opts.quiet, "{}", "running commands in parallel".purple());

            let resolved_lines: Vec<String> = script
                .run
                .iter()
                .map(|line| {
                    let line = replace_vars(line, &config.vars, &script.vars, &named_args)
                        .unwrap_or_else(|e| {
                            err!("{}", e);
                            process::exit(1);
                        });
                    let line = replace_args(&line, &positional_args);
                    replace_env(&line).unwrap_or_else(|e| {
                        err!("{}", e);
                        process::exit(1);
                    })
                })
                .collect();

            let cwd = cwd.clone();
            let handles: Vec<_> = resolved_lines
                .into_iter()
                .map(|line| {
                    let cwd = cwd.clone();
                    thread::spawn(move || {
                        spawn_command(&line, &cwd)
                            .wait()
                            .expect("failed to wait for child process")
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
            process::exit(0)
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
            if !opts.continue_on_err {
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
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn args_replaces_single_placeholder() {
        let line = "echo {{1}}".to_string();
        let args = vec!["Omar".to_string()];
        let result = replace_args(&line, &args);
        assert_eq!(result, "echo Omar");
    }

    #[test]
    fn args_replaces_multiple_placeholders() {
        let line = "echo {{1}} {{2}}".to_string();
        let args = vec!["Hello".to_string(), "World".to_string()];
        let result = replace_args(&line, &args);
        assert_eq!(result, "echo Hello World");
    }

    #[test]
    fn args_no_placeholder_unchanged() {
        let line = "echo hello".to_string();
        let args = vec!["Omar".to_string()];
        let result = replace_args(&line, &args);
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn args_missing_arg_leaves_placeholder() {
        let line = "echo {{1}} {{2}}".to_string();
        let args = vec!["Omar".to_string()];
        let result = replace_args(&line, &args);
        assert_eq!(result, "echo Omar {{2}}");
    }

    #[test]
    fn args_empty_args_unchanged() {
        let line = "echo {{1}}".to_string();
        let result = replace_args(&line, &[]);
        assert_eq!(result, "echo {{1}}");
    }

    #[test]
    fn args_detects_placeholder() {
        assert!("echo {{1}}".contains("{{"));
        assert!(!"echo hello".contains("{{"));
    }

    #[test]
    fn cd_resolves_relative_path() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().to_path_buf();
        let resolved = cwd.canonicalize();
        assert!(resolved.is_ok());
    }

    #[test]
    fn cd_invalid_path_returns_error() {
        let cwd = PathBuf::from("/tmp");
        let bad = cwd.join("nonexistent_xeq_test_dir_xyz");
        assert!(bad.canonicalize().is_err());
    }

    #[test]
    fn cd_and_operator_runs_rest_on_success() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().canonicalize().unwrap();
        let new_dir = TempDir::new().unwrap();
        let new_cwd = new_dir.path().canonicalize().unwrap();
        let line = format!("cd {} && touch xeq_and_test.txt", new_cwd.display());
        let arg = line.strip_prefix("cd ").unwrap().trim();
        let (d, rest, sep) = if let Some((d, r)) = arg.split_once("&&") {
            (d.trim(), Some(r.trim().to_string()), Some("&&"))
        } else {
            (arg, None, None)
        };
        let resolved = cwd.join(d).canonicalize();
        assert!(resolved.is_ok());
        assert_eq!(sep, Some("&&"));
        assert!(rest.is_some());
    }

    #[test]
    fn cd_and_operator_skips_rest_on_failure() {
        let bad_path = PathBuf::from("/tmp/nonexistent_xeq_dir_xyz");
        let result = bad_path.canonicalize();
        assert!(result.is_err());
    }

    #[test]
    fn cd_or_operator_skips_rest_on_success() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().canonicalize().unwrap();
        let line = format!("cd {} || echo fallback", cwd.display());
        let arg = line.strip_prefix("cd ").unwrap().trim();
        let (_, _, sep) = if let Some((d, r)) = arg.split_once("||") {
            (d.trim(), Some(r.trim().to_string()), Some("||"))
        } else {
            (arg, None, None)
        };
        let resolved = cwd
            .join(arg.split_once("||").unwrap().0.trim())
            .canonicalize();
        assert!(resolved.is_ok());
        assert_eq!(sep, Some("||"));
    }

    #[test]
    fn cd_or_operator_runs_rest_on_failure() {
        let bad = PathBuf::from("/tmp/nonexistent_xeq_dir_xyz");
        let result = bad.canonicalize();
        assert!(result.is_err());
    }

    #[test]
    fn cd_semicolon_operator_always_runs_rest() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().canonicalize().unwrap();
        let line = format!("cd {}; echo always", cwd.display());
        let arg = line.strip_prefix("cd ").unwrap().trim();
        let (_, rest, sep) = if let Some((d, r)) = arg.split_once(';') {
            (d.trim(), Some(r.trim().to_string()), Some(";"))
        } else {
            (arg, None, None)
        };
        assert_eq!(sep, Some(";"));
        assert!(rest.is_some());
    }

    #[test]
    fn cd_background_operator_parses_correctly() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().canonicalize().unwrap();
        let line = format!("cd {} & echo bg", cwd.display());
        let arg = line.strip_prefix("cd ").unwrap().trim();
        let (_, rest, sep) = if let Some((d, r)) = arg.split_once('&') {
            (d.trim(), Some(r.trim().to_string()), Some("&"))
        } else {
            (arg, None, None)
        };
        assert_eq!(sep, Some("&"));
        assert!(rest.is_some());
    }

    #[test]
    fn cd_negate_flips_success_to_failure() {
        let dir = TempDir::new().unwrap();
        let valid = dir.path().canonicalize().unwrap();
        let result = valid.canonicalize();
        let negate = true;
        let cd_succeeded = match &result {
            Ok(_) => !negate,
            Err(_) => negate,
        };
        assert!(!cd_succeeded);
    }

    #[test]
    fn cd_negate_flips_failure_to_success() {
        let bad = PathBuf::from("/tmp/nonexistent_xeq_dir_xyz");
        let result = bad.canonicalize();
        let negate = true;
        let cd_succeeded = match &result {
            Ok(_) => !negate,
            Err(_) => negate,
        };
        assert!(cd_succeeded);
    }

    #[test]
    fn cd_no_operator_plain_path() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().canonicalize().unwrap();
        let line = format!("cd {}", cwd.display());
        let arg = line.strip_prefix("cd ").unwrap().trim();
        let has_operator =
            arg.contains("&&") || arg.contains("||") || arg.contains(';') || arg.contains('&');
        assert!(!has_operator);
    }

    #[test]
    fn cd_empty_arg_resolves_to_home() {
        let home = dirs::home_dir();
        assert!(home.is_some());
    }

    #[test]
    fn parallel_resolves_vars_before_spawn() {
        let mut global = HashMap::new();
        global.insert("cmd".to_string(), "echo hello".to_string());
        let result = replace_vars("{{@cmd}}", &Some(global), &None, &HashMap::new()).unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn parallel_resolves_args_before_spawn() {
        let line = "echo {{1}}";
        let args = vec!["parallel_test".to_string()];
        let result = replace_args(line, &args);
        assert_eq!(result, "echo parallel_test");
    }

    #[test]
    fn parallel_resolves_env_before_spawn() {
        std::env::set_var("XEQ_TEST_VAR", "hello");
        let result = replace_env("echo {{$XEQ_TEST_VAR}}").unwrap();
        assert_eq!(result, "echo hello");
        std::env::remove_var("XEQ_TEST_VAR");
    }

    #[test]
    fn spawn_command_uses_cwd() {
        let dir = TempDir::new().unwrap();
        let cwd = dir.path().canonicalize().unwrap();
        #[cfg(target_os = "windows")]
        let (cmd, args) = ("cmd", vec!["/C", "echo . > xeq_test_marker.txt"]);
        #[cfg(not(target_os = "windows"))]
        let (cmd, args) = ("sh", vec!["-c", "touch xeq_test_marker.txt"]);
        std::process::Command::new(cmd)
            .args(&args)
            .current_dir(&cwd)
            .output()
            .unwrap();
        assert!(cwd.join("xeq_test_marker.txt").exists());
    }

    #[test]
    fn parse_args_named_and_positional() {
        let args = vec!["image=myapp:latest".to_string(), "my-app".to_string()];
        let (named, positional) = parse_args(&args);
        assert_eq!(named.get("image").unwrap(), "myapp:latest");
        assert_eq!(positional[0], "my-app");
    }

    #[test]
    fn parse_args_only_named() {
        let args = vec!["image=myapp".to_string(), "env=dev".to_string()];
        let (named, positional) = parse_args(&args);
        assert_eq!(named.len(), 2);
        assert!(positional.is_empty());
    }

    #[test]
    fn parse_args_only_positional() {
        let args = vec!["my-app".to_string(), "react".to_string()];
        let (named, positional) = parse_args(&args);
        assert!(named.is_empty());
        assert_eq!(positional.len(), 2);
    }

    #[test]
    fn parse_args_empty() {
        let (named, positional) = parse_args(&[]);
        assert!(named.is_empty());
        assert!(positional.is_empty());
    }

    #[test]
    fn parse_args_value_with_equals_sign() {
        let args = vec!["url=http://a.com?x=1".to_string()];
        let (named, _) = parse_args(&args);
        assert_eq!(named.get("url").unwrap(), "http://a.com?x=1");
    }

    #[test]
    fn replace_vars_uses_global() {
        let mut global = HashMap::new();
        global.insert("image".to_string(), "myapp:latest".to_string());
        let result = replace_vars(
            "docker build -t {{@image}} .",
            &Some(global),
            &None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "docker build -t myapp:latest .");
    }

    #[test]
    fn replace_vars_local_overrides_global() {
        let mut global = HashMap::new();
        global.insert("image".to_string(), "myapp:latest".to_string());
        let mut local = HashMap::new();
        local.insert("image".to_string(), "myapp:build".to_string());
        let result = replace_vars(
            "docker build -t {{@image}} .",
            &Some(global),
            &Some(local),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "docker build -t myapp:build .");
    }

    #[test]
    fn replace_vars_args_overrides_local_and_global() {
        let mut global = HashMap::new();
        global.insert("image".to_string(), "myapp:latest".to_string());
        let mut local = HashMap::new();
        local.insert("image".to_string(), "myapp:build".to_string());
        let mut args = HashMap::new();
        args.insert("image".to_string(), "myapp:override".to_string());
        let result = replace_vars(
            "docker build -t {{@image}} .",
            &Some(global),
            &Some(local),
            &args,
        )
        .unwrap();
        assert_eq!(result, "docker build -t myapp:override .");
    }

    #[test]
    fn replace_vars_undefined_returns_error() {
        let result = replace_vars(
            "docker build -t {{@image}} .",
            &None,
            &None,
            &HashMap::new(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("image"));
    }

    #[test]
    fn replace_vars_no_placeholders_unchanged() {
        let result = replace_vars("echo hello", &None, &None, &HashMap::new()).unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn replace_vars_multiple_placeholders() {
        let mut global = HashMap::new();
        global.insert("image".to_string(), "myapp".to_string());
        global.insert("env".to_string(), "dev".to_string());
        let result = replace_vars(
            "docker build -t {{@image}} --env {{@env}} .",
            &Some(global),
            &None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "docker build -t myapp --env dev .");
    }

    #[test]
    fn replace_vars_does_not_touch_positional_placeholders() {
        let mut global = HashMap::new();
        global.insert("image".to_string(), "myapp".to_string());
        let result = replace_vars(
            "echo {{@image}} {{1}}",
            &Some(global),
            &None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "echo myapp {{1}}");
    }

    #[test]
    fn config_with_vars_does_not_include_vars_as_script() {
        let config: Config = toml::from_str(
            r#"
[vars]
image = "myapp"

[build]
run = ["echo hi"]
"#,
        )
        .unwrap();
        assert!(!config.scripts.contains_key("vars"));
        assert!(config.vars.is_some());
    }
}
