use std::{collections::HashSet, env, path::PathBuf, process, thread};

use crate::types::ScriptOption::*;
use crate::types::Scripts;

#[derive(Clone, Copy)]
pub struct RunOptions {
    pub continue_on_err: bool,
    pub clear: bool,
    pub quiet: bool,
    pub parallel: bool,
    pub allow_recursion: bool,
}

pub fn replace_args(line: &str, args: &[String]) -> String {
    let mut line = line.to_owned();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{{{}}}}}", i + 1);
        line = line.replace(&placeholder, arg);
    }
    line
}

pub fn spawn_command(line: &str) -> std::process::Child {
    #[cfg(target_os = "windows")]
    return std::process::Command::new("cmd")
        .args(["/C", line])
        .spawn()
        .expect("failed to spawn");

    #[cfg(not(target_os = "windows"))]
    return std::process::Command::new("sh")
        .args(["-c", line])
        .spawn()
        .expect("failed to spawn");
}

pub fn run(
    script_name: String,
    scripts: &Scripts,
    visited: &mut HashSet<String>,
    args: Option<Vec<String>>,
    mut opts: RunOptions,
) {
    let script = match scripts.get(&script_name) {
        Some(x) => x,
        None => {
            err!("Script '{}' not found.", script_name);
            if !opts.continue_on_err {
                process::exit(1);
            } else {
                return;
            }
        }
    };

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
    }

    if opts.parallel {
        log!(opts.quiet, "{}", "running in parallel".purple());
        let handles: Vec<_> = script
            .run
            .iter()
            .filter(|line| {
                if line.starts_with("cd ") {
                    err!("'cd' is not supported in parallel mode, skipping: {}", line);
                    return false;
                }
                if line.starts_with("xeq://") {
                    err!(
                        "nested scripts are not supported in parallel mode, skipping: {}",
                        line
                    );
                    return false;
                }
                true
            })
            .map(|line| {
                let line = line.clone();
                thread::spawn(move || spawn_command(&line).wait().expect("Failed to wait"))
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        return;
    }

    let total = script.run.len();
    for (i, line) in script.run.iter().enumerate() {
        if opts.clear {
            clearscreen::clear().unwrap();
        }
        let mut line = line.clone();

        if line.contains("{{") && args.is_none() {
            err!(
                "Script '{}' expects arguments but none were provided.",
                script_name
            );
            std::process::exit(1);
        } else if let Some(ref args) = args {
            line = replace_args(&line, args);
        }

        log!(opts.quiet, "[{}/{}] {}", i + 1, total, line.yellow());
        if let Some(name) = line.strip_prefix("xeq://") {
            let name = name.to_owned();
            if visited.contains(&name) && !opts.allow_recursion {
                err!("Circular dependency detected: '{}'", script_name);
                std::process::exit(1);
            }
            visited.insert(name.clone());
            log!(
                opts.quiet,
                "Calling script \'{}\'----------------",
                name.purple()
            );
            run(name.clone(), scripts, visited, args.clone(), opts);
            visited.remove(&name);
            continue;
        }
        if let Some(arg) = line.strip_prefix("cd ") {
            let arg = arg.trim();
            let path = match arg.is_empty() {
                true => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
                false => PathBuf::from(arg),
            };
            if let Err(e) = env::set_current_dir(&path) {
                err!("cd: {}: {}", path.display(), e);
                if !opts.continue_on_err {
                    std::process::exit(1);
                }
            } else {
                log!(
                    opts.quiet,
                    "Changing directory to {}",
                    path.display().to_string().yellow()
                );
            }
            continue;
        }

        let status = spawn_command(&line).wait().expect("Failed to wait");

        if !status.success() {
            err!(
                "Command failed with exit code {}.",
                status.code().unwrap_or(-1)
            );
            if !opts.continue_on_err {
                std::process::exit(status.code().unwrap_or(1));
            }
        } else {
            log!(opts.quiet, "{}", "Done".green());
        }
    }

    log!(
        opts.quiet,
        "{} \'{}\' {}----------------",
        "script",
        script_name,
        "commands are completed".green().bold()
    );
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
    fn cd_set_current_dir_valid_path() {
        let dir = TempDir::new().unwrap();
        assert!(std::env::set_current_dir(dir.path()).is_ok());
    }

    #[test]
    fn cd_set_current_dir_invalid_path() {
        assert!(std::env::set_current_dir("/nonexistent/path/xyz").is_err());
    }
}
