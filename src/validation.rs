use colored::Colorize;

use crate::types::{Config, ScriptOption, Scripts};

fn check_recursion(
    name: &str,
    scripts: &Scripts,
    visited: &mut Vec<String>,
    has_errors: &mut bool,
    script_has_errors: &mut bool,
) {
    for cmd in &scripts[name].run {
        if let Some(target) = cmd.strip_prefix("xeq://") {
            if visited.contains(&target.to_string()) {
                err!(
                    "'{}': circular dependency detected — '{}' is already in the call chain: {}",
                    name,
                    target,
                    visited.join(" -> ")
                );
                *has_errors = true;
                *script_has_errors = true;
                return;
            }
            if scripts.contains_key(target) {
                visited.push(target.to_string());
                check_recursion(target, scripts, visited, has_errors, script_has_errors);
                visited.pop();
            }
        }
    }
}

pub fn validate(config: &Config) -> bool {
    let mut has_errs = false;
    let scripts = &config.scripts;

    for (name, script) in scripts {
        let mut script_has_errs = false;
        log!(false, "validating '{}'", name);

        let mut visited = vec![name.clone()];
        check_recursion(name, scripts, &mut visited, &mut has_errs, &mut false);

        if let Some(opts) = &script.options {
            if opts.contains(&ScriptOption::Parallel) {
                let has_cd = script.run.iter().any(|l| l.starts_with("cd "));
                let has_nested = script.run.iter().any(|l| l.starts_with("xeq://"));
                if has_cd {
                    err!("'{}': has 'parallel' but contains a 'cd' command", name);
                    has_errs = true;
                    script_has_errs = true;
                }
                if has_nested {
                    err!(
                        "'{}': has 'parallel' but contains nested 'xeq://' calls",
                        name
                    );
                    has_errs = true;
                    script_has_errs = true;
                }
            }
        }

        if let Some(dir) = &script.dir {
            if !std::path::Path::new(dir).exists() {
                err!("'{}': dir '{}' does not exist", name, dir);
                has_errs = true;
                script_has_errs = true;
            }
        }

        for cmd in &script.run {
            if let Some(target) = cmd.strip_prefix("xeq://") {
                if !scripts.contains_key(target) {
                    err!(
                        "'{}': calls 'xeq://{}' but that script doesn't exist",
                        name,
                        target
                    );
                    has_errs = true;
                    script_has_errs = true;
                }
            }

            let mut i = 0;
            while let Some(start) = cmd[i..].find("{{@") {
                let start = i + start;
                if let Some(end) = cmd[start..].find("}}") {
                    let key = &cmd[start + 3..start + end];
                    let in_global = config.vars.as_ref().is_some_and(|v| v.contains_key(key));
                    let in_local = script.vars.as_ref().is_some_and(|v| v.contains_key(key));
                    if !in_global && !in_local {
                        println!("{} '{}': '{{{{@{}}}}}' is not defined in vars — must be passed at runtime with --args",
                            "[xeq]".yellow().bold(), name, key);
                    }
                    i = start + end + 2;
                } else {
                    break;
                }
            }
        }

        if !script_has_errs {
            log!(
                false,
                "{} '{}' {}\n",
                "script".green(),
                name,
                "passed".green()
            );
        }
        println!("")
    }

    has_errs
}
