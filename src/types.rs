use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct Config {
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
    Parallel,
    ContinueOnErr,
    AllowRecursion,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub description: Option<String>,
    pub options: Option<Vec<ScriptOption>>,
    pub run: Vec<String>,
    pub vars: Option<HashMap<String, String>>,
}

pub type Scripts = HashMap<String, Script>;

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_toml() -> &'static str {
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
    fn script_struct_serializes_correctly() {
        let script = Script {
            description: None,
            options: None,
            vars: None,
            run: vec!["echo hi".to_string(), "echo bye".to_string()],
        };
        let toml_str = toml::to_string(&script).unwrap();
        assert!(toml_str.contains("echo hi"));
        assert!(toml_str.contains("echo bye"));
    }

    #[test]
    fn script_struct_deserializes_correctly() {
        let script: Script = toml::from_str(r#"run = ["echo hello"]"#).unwrap();
        assert_eq!(script.run.len(), 1);
        assert_eq!(script.run[0], "echo hello");
    }

    #[test]
    fn parse_valid_scripts_toml() {
        let scripts: Scripts = toml::from_str(valid_toml()).unwrap();
        assert!(scripts.contains_key("build"));
        assert!(scripts.contains_key("test"));
        assert!(scripts.contains_key("empty"));
    }

    #[test]
    fn parse_script_commands_are_correct() {
        let scripts: Scripts = toml::from_str(valid_toml()).unwrap();
        let build = scripts.get("build").unwrap();
        assert_eq!(build.run.len(), 2);
        assert_eq!(build.run[0], "echo building");
        assert_eq!(build.run[1], "echo done");
    }

    #[test]
    fn parse_empty_run_array() {
        let scripts: Scripts = toml::from_str(valid_toml()).unwrap();
        assert_eq!(scripts.get("empty").unwrap().run.len(), 0);
    }

    #[test]
    fn parse_invalid_toml_returns_error() {
        let result: Result<Scripts, _> = toml::from_str("{ not valid toml");
        assert!(result.is_err());
    }

    #[test]
    fn parse_wrong_schema_missing_run_field() {
        let result: Result<Scripts, _> = toml::from_str(
            r#"
[build]
commands = ["echo hi"]
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_toml_object() {
        let scripts: Scripts = toml::from_str("").unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn parse_script_with_cd_command() {
        let scripts: Scripts = toml::from_str(
            r#"
[setup]
run = ["cd /tmp", "echo hello"]
"#,
        )
        .unwrap();
        assert_eq!(scripts.get("setup").unwrap().run[0], "cd /tmp");
    }

    #[test]
    fn script_lookup_existing_key() {
        let scripts: Scripts = toml::from_str(valid_toml()).unwrap();
        assert!(scripts.get("build").is_some());
    }

    #[test]
    fn script_lookup_missing_key() {
        let scripts: Scripts = toml::from_str(valid_toml()).unwrap();
        assert!(scripts.get("nonexistent").is_none());
    }

    #[test]
    fn script_lookup_case_sensitive() {
        let scripts: Scripts = toml::from_str(valid_toml()).unwrap();
        assert!(scripts.get("Build").is_none());
        assert!(scripts.get("build").is_some());
    }

    #[test]
    fn script_chaining_target_exists() {
        let scripts: Scripts = toml::from_str(
            r#"
[build]
run = ["xeq://setup", "cargo build"]

[setup]
run = ["echo setting up"]
"#,
        )
        .unwrap();
        let build = scripts.get("build").unwrap();
        let chain_target = &build.run[0]["xeq://".len()..];
        assert!(scripts.get(chain_target).is_some());
    }

    #[test]
    fn script_chaining_target_missing() {
        let scripts: Scripts = toml::from_str(
            r#"
[build]
run = ["xeq://nonexistent"]
"#,
        )
        .unwrap();
        let build = scripts.get("build").unwrap();
        let chain_target = &build.run[0]["xeq://".len()..];
        assert!(scripts.get(chain_target).is_none());
    }

    #[test]
    fn xeq_chaining_target_missing() {
        let scripts: Scripts = toml::from_str(
            r#"
[build]
run = ["xeq://nonexistent"]
"#,
        )
        .unwrap();
        let chain_target = &"xeq://nonexistent"["xeq://".len()..];
        assert!(scripts.get(chain_target).is_none());
    }
}
