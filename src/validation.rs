use crate::types::{Config, Script, ScriptOption, Scripts};
use which::which;

fn check_recursion(
    name: &str,
    scripts: &Scripts,
    visited: &mut Vec<String>,
    has_errors: &mut bool,
    script_has_errors: &mut bool,
) {
    for cmd in &scripts[name].run {
        if let Some(target) = cmd.strip_prefix("xeq:") {
            if visited.contains(&target.to_string()) {
                err!(
                    "'{}': circular dependency detected, '{}' is already in the call chain: {}",
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

fn check_cmds(
    name: &str,
    cmds: &[String],
    scripts: &Scripts,
    config: &Config,
    script: &Script,
    has_errs: &mut bool,
    script_has_errs: &mut bool,
    runtime: bool,
) {
    for cmd in cmds {
        if let Some(target) = cmd.strip_prefix("xeq:") {
            if !scripts.contains_key(target) {
                err!(
                    "'{}': calls 'xeq:{}' but that script doesn't exist",
                    name,
                    target
                );
                *has_errs = true;
                *script_has_errs = true;
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
                    log!(
                        false,
                        "'{}': '{{{{@{}}}}}' is not defined in vars, must be passed at runtime with --args",
                        name,
                        key
                    );
                }
                i = start + end + 2;
            } else {
                break;
            }
        }

        if runtime {
            while let Some(start) = cmd[i..].find("{{$") {
                let start = i + start;
                if let Some(end) = cmd[start..].find("}}") {
                    let key = &cmd[start + 3..start + end];
                    if std::env::var(key).is_err() {
                        err!("'{}': '{{{{${}}}}}' is not set", name, key);
                        *has_errs = true;
                        *script_has_errs = true;
                    }
                    i = start + end + 2;
                } else {
                    break;
                }
            }

            let command_name = cmd.split_whitespace().next().unwrap_or_default();
            if !command_name.starts_with("xeq:") && which(command_name).is_err() {
                err!("'{}': '{}' command doesn't exist", name, command_name);
                *has_errs = true;
                *script_has_errs = true;
            }
        }
    }
}

pub fn validate(config: &Config, runtime: bool) -> bool {
    if runtime {
        dotenvy::dotenv().ok();
    }

    let mut has_errs = false;
    let scripts = &config.scripts;

    for (name, script) in scripts {
        let mut script_has_errs = false;
        log!(false, "validating '{}'", name);
        let events = script.on_error.is_some() || script.on_success.is_some();

        let mut visited = vec![name.clone()];
        check_recursion(
            name,
            scripts,
            &mut visited,
            &mut has_errs,
            &mut script_has_errs,
        );

        if let Some(s) = &config.default {
            if scripts.get(s).is_none() {
                err!("The default script \'{s}\' doesn't exist");
            }
        }

        if events
            && script
                .options
                .as_ref()
                .is_some_and(|o| o.contains(&ScriptOption::ContinueOnErr))
        {
            err!(
                "'{}': events and continue_on_err cannot be used together",
                name
            );
            has_errs = true;
            script_has_errs = true;
        }

        if let Some(n) = script.parallel_threads {
            if n <= 1 {
                err!(
                    "'{}': parallel_threads should be greater than 1 to run in parallel",
                    name
                );
                has_errs = true;
                script_has_errs = true;
            }
        }

        if script.parallel_threads.is_some() {
            let has_cd = script.run.iter().any(|l| l.starts_with("cd "));
            let has_nested = script.run.iter().any(|l| l.starts_with("xeq:"));
            if has_cd {
                err!("'{}': has 'parallel' but contains a 'cd' command", name);
                has_errs = true;
                script_has_errs = true;
            }
            if has_nested {
                err!(
                    "'{}': has 'parallel' but contains nested 'xeq:' calls",
                    name
                );
                has_errs = true;
                script_has_errs = true;
            }
        }

        if let Some(dir) = &script.dir {
            if !std::path::Path::new(dir).exists() {
                err!("'{}': dir '{}' does not exist", name, dir);
                has_errs = true;
                script_has_errs = true;
            } else if runtime {
                let original = std::env::current_dir().ok();
                if std::env::set_current_dir(dir).is_err() {
                    err!("'{}': cannot cd into dir '{}'", name, dir);
                    has_errs = true;
                    script_has_errs = true;
                }
                if let Some(orig) = original {
                    std::env::set_current_dir(orig).ok();
                }
            }
        }

        check_cmds(
            name,
            &script.run,
            scripts,
            config,
            script,
            &mut has_errs,
            &mut script_has_errs,
            runtime,
        );

        if let Some(cmds) = &script.on_error {
            check_cmds(
                name,
                cmds,
                scripts,
                config,
                script,
                &mut has_errs,
                &mut script_has_errs,
                runtime,
            );
        }

        if let Some(cmds) = &script.on_success {
            check_cmds(
                name,
                cmds,
                scripts,
                config,
                script,
                &mut has_errs,
                &mut script_has_errs,
                runtime,
            );
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
        println!();
    }

    has_errs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Config, Script, ScriptOption};
    use std::collections::HashMap;

    fn make_config(scripts: Scripts) -> Config {
        Config {
            default: None,
            shell: None,
            vars: None,
            scripts,
        }
    }

    fn simple_script(run: Vec<&str>) -> Script {
        Script {
            description: None,
            options: None,
            parallel_threads: None,
            on_error: None,
            on_success: None,
            dir: None,
            vars: None,
            run: run.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn valid_script_passes() {
        let mut scripts = HashMap::new();
        scripts.insert("build".into(), simple_script(vec!["echo hi"]));
        let config = make_config(scripts);
        assert!(!validate(&config, false));
    }

    #[test]
    fn missing_nested_script_is_an_error() {
        let mut scripts = HashMap::new();
        scripts.insert("build".into(), simple_script(vec!["xeq:nonexistent"]));
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn direct_self_call_is_circular() {
        let mut scripts = HashMap::new();
        scripts.insert("build".into(), simple_script(vec!["xeq:build"]));
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn indirect_circular_dependency_is_caught() {
        let mut scripts = HashMap::new();
        scripts.insert("a".into(), simple_script(vec!["xeq:b"]));
        scripts.insert("b".into(), simple_script(vec!["xeq:a"]));
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn linear_chain_passes() {
        let mut scripts = HashMap::new();
        scripts.insert("a".into(), simple_script(vec!["xeq:b"]));
        scripts.insert("b".into(), simple_script(vec!["xeq:c"]));
        scripts.insert("c".into(), simple_script(vec!["echo done"]));
        let config = make_config(scripts);
        assert!(!validate(&config, false));
    }

    #[test]
    fn fallback_and_continue_on_err_together_is_an_error() {
        let mut scripts = HashMap::new();
        scripts.insert(
            "build".into(),
            Script {
                on_error: Some(vec!["notify".into()]),
                options: Some(vec![ScriptOption::ContinueOnErr]),
                ..simple_script(vec!["cargo build"])
            },
        );
        scripts.insert("notify".into(), simple_script(vec!["echo failed"]));
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn parallel_threads_of_one_is_an_error() {
        let mut scripts = HashMap::new();
        scripts.insert(
            "check".into(),
            Script {
                parallel_threads: Some(1),
                ..simple_script(vec!["cargo test"])
            },
        );
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn parallel_with_cd_is_an_error() {
        let mut scripts = HashMap::new();
        scripts.insert(
            "check".into(),
            Script {
                parallel_threads: Some(4),
                ..simple_script(vec!["cd /tmp", "cargo test"])
            },
        );
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn parallel_with_nested_call_is_an_error() {
        let mut scripts = HashMap::new();
        scripts.insert(
            "check".into(),
            Script {
                parallel_threads: Some(4),
                ..simple_script(vec!["xeq:setup", "cargo test"])
            },
        );
        scripts.insert("setup".into(), simple_script(vec!["echo setup"]));
        let config = make_config(scripts);
        assert!(validate(&config, false));
    }

    #[test]
    fn valid_nested_call_passes() {
        let mut scripts = HashMap::new();
        scripts.insert("deploy".into(), simple_script(vec!["xeq:build"]));
        scripts.insert("build".into(), simple_script(vec!["cargo build"]));
        let config = make_config(scripts);
        assert!(!validate(&config, false));
    }
}