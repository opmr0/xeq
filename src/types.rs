use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub shell: Option<String>,
    pub vars: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub scripts: Scripts,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SavedPath {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptOption {
    Quiet,
    Clear,
    ContinueOnErr,
    AllowRecursion,
    Summary,
    AllowEmptyVars,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub description: Option<String>,
    pub options: Option<Vec<ScriptOption>>,
    pub parallel_threads: Option<usize>,
    pub fallback: Option<String>,
    pub dir: Option<String>,
    pub run: Vec<String>,
    pub vars: Option<HashMap<String, String>>,
}

pub type Scripts = HashMap<String, Script>;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_toml() -> &'static str {
        r#"
[build]
run = ["echo building", "echo done"]

[test]
run = ["echo testing"]

[empty]
run = []
"#
    }

    #[test]
    fn parses_scripts_correctly() {
        let scripts: Scripts = toml::from_str(sample_toml()).unwrap();
        assert!(scripts.contains_key("build"));
        assert!(scripts.contains_key("test"));
        assert!(scripts.contains_key("empty"));
    }

    #[test]
    fn parses_run_commands() {
        let scripts: Scripts = toml::from_str(sample_toml()).unwrap();
        let build = scripts.get("build").unwrap();
        assert_eq!(build.run, vec!["echo building", "echo done"]);
    }

    #[test]
    fn parses_empty_run_array() {
        let scripts: Scripts = toml::from_str(sample_toml()).unwrap();
        assert!(scripts.get("empty").unwrap().run.is_empty());
    }

    #[test]
    fn missing_run_field_is_an_error() {
        let result: Result<Scripts, _> = toml::from_str("[build]\ncommands = [\"echo hi\"]");
        assert!(result.is_err());
    }

    #[test]
    fn empty_toml_gives_empty_scripts() {
        let scripts: Scripts = toml::from_str("").unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn script_lookup_is_case_sensitive() {
        let scripts: Scripts = toml::from_str(sample_toml()).unwrap();
        assert!(scripts.get("Build").is_none());
        assert!(scripts.get("build").is_some());
    }

    #[test]
    fn script_serializes_run_commands() {
        let script = Script {
            dir: None,
            fallback: None,
            description: None,
            parallel_threads: None,
            options: None,
            vars: None,
            run: vec!["echo hi".into(), "echo bye".into()],
        };
        let out = toml::to_string(&script).unwrap();
        assert!(out.contains("echo hi"));
        assert!(out.contains("echo bye"));
    }

    #[test]
    fn nested_script_target_exists() {
        let scripts: Scripts = toml::from_str(
            r#"
[build]
run = ["xeq://setup", "cargo build"]

[setup]
run = ["echo setting up"]
"#,
        )
        .unwrap();
        let target = &scripts["build"].run[0]["xeq://".len()..];
        assert!(scripts.contains_key(target));
    }

    #[test]
    fn nested_script_target_missing() {
        let scripts: Scripts = toml::from_str(
            r#"
[build]
run = ["xeq://nonexistent"]
"#,
        )
        .unwrap();
        let target = &scripts["build"].run[0]["xeq://".len()..];
        assert!(!scripts.contains_key(target));
    }

    #[test]
    fn script_with_all_fields_parses() {
        let scripts: Scripts = toml::from_str(
            r#"
[deploy]
description = "Deploy to production"
parallel_threads = 4
fallback = "notify"
dir = "/tmp"
options = ["quiet", "continue_on_err"]
vars = { env = "prod" }
run = ["echo deploying"]
"#,
        )
        .unwrap();
        let s = scripts.get("deploy").unwrap();
        assert_eq!(s.description.as_deref(), Some("Deploy to production"));
        assert_eq!(s.parallel_threads, Some(4));
        assert_eq!(s.fallback.as_deref(), Some("notify"));
        assert!(s.vars.as_ref().unwrap().contains_key("env"));
    }
}
