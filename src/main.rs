use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{env, io, process};

const PREFIX: &str = "[xeq]";

macro_rules! log {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!("{} {}", PREFIX.cyan().bold(), format!($($arg)*));
        }
    };
}

macro_rules! err {
    ($($arg:tt)*) => {
        eprintln!("{} {}", PREFIX.red().bold(), format!($($arg)*).red());
    };
}

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
        script: String,
        #[arg(short = 'C', long, help = "Keep running even if a command fails")]
        continue_on_err: bool,
        #[arg(short, long, help = "Clear the screen between commands")]
        clear: bool,
        #[arg(short, long, help = "Suppress xeq output")]
        quiet: bool,
    },
    List,
}

#[derive(Serialize, Deserialize, Default)]
struct SavedPath {
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct Script {
    run: Vec<String>,
}

type Scripts = HashMap<String, Script>;

fn validate_path(path: &PathBuf) -> Result<PathBuf, io::Error> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{:?} does not exist", path),
        ));
    } else if path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::IsADirectory,
            format!("{:?} is not a file", path),
        ));
    } else if !is_json(path) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{:?} is not a JSON file", path),
        ));
    }
    Ok(path.clone())
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

fn save_path(path: PathBuf) -> Result<(), io::Error> {
    validate_path(&path)?;
    let config = SavedPath {
        path: path.canonicalize().unwrap(),
    };
    confy::store("xeq", "path", &config).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to store config: {}", e),
        )
    })
}

fn load_path() -> Option<PathBuf> {
    let config: SavedPath = confy::load("xeq", "path").ok()?;
    if config.path.as_os_str().is_empty() {
        return None;
    }
    Some(config.path)
}

fn read_scripts() -> Result<Scripts, io::Error> {
    let path = load_path().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, ""))?;

    let file = File::open(&path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot open file at {}: {}", path.display(), e),
        )
    })?;

    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {}", e)))
}

fn validate_or_exit() {
    if let Some(path) = load_path() {
        if validate_path(&path).is_err() {
            err!(
                "The commands JSON file has been deleted or moved.\n      Configure xeq using: xeq config <path/to/file.json>"
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
                        PREFIX.cyan().bold(),
                        "Configuration saved successfully!".green()
                    );
                }
            } else {
                validate_or_exit();
                let path = match load_path() {
                    Some(x) => x,
                    None => {
                        err!(
                            "xeq is not configured.\n      Configure it using: xeq config <path/to/file.json>"
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
            script,
            continue_on_err,
            clear,
            quiet,
        } => {
            validate_or_exit();

            let scripts = match read_scripts() {
                Ok(x) => x,
                Err(e) => {
                    err!("{}", e);
                    process::exit(1);
                }
            };

            let script = match scripts.get(&script) {
                Some(x) => x,
                None => {
                    err!("Script '{}' not found.", script);
                    process::exit(1);
                }
            };

            let total = script.run.len();
            for (i, line) in script.run.iter().enumerate() {
                if clear {
                    clearscreen::clear().unwrap();
                }

                log!(quiet, "[{}/{}] {}", i + 1, total, line.yellow());

                if line.starts_with("cd ") {
                    let arg = line[3..].trim();
                    let path = match arg.is_empty() {
                        true => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
                        false => PathBuf::from(arg),
                    };
                    if let Err(e) = env::set_current_dir(&path) {
                        err!("cd: {}: {}", path.display(), e);
                        if !continue_on_err {
                            process::exit(1);
                        }
                    } else {
                        log!(quiet, "cd {}", path.display().to_string().yellow());
                    }
                    continue;
                }

                #[cfg(target_os = "windows")]
                let mut cmd = std::process::Command::new("cmd");
                #[cfg(target_os = "windows")]
                cmd.args(["/C", line]);

                #[cfg(not(target_os = "windows"))]
                let mut cmd = std::process::Command::new("sh");
                #[cfg(not(target_os = "windows"))]
                cmd.args(["-c", line]);

                let status = cmd
                    .spawn()
                    .expect("failed to spawn")
                    .wait()
                    .expect("failed to wait");

                if !status.success() {
                    err!(
                        "Command failed with exit code {}.",
                        status.code().unwrap_or(-1)
                    );
                    if !continue_on_err {
                        process::exit(status.code().unwrap_or(1));
                    }
                } else {
                    log!(quiet, "{}", "Done".green());
                }
            }

            log!(quiet, "{}", "All commands completed".green().bold());
        }
        Command::List => {
            validate_or_exit();
            log!(false, "Listing tasks... \n");
            let content = read_scripts().unwrap();
            for s in content {
                println!("{} runs:", s.0.cyan());
                for c in s.1.run.iter() {
                    println!("\t{}", c.yellow())
                }
            }
        }
    }
}

//tests ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp_json(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("xeq.json");
        fs::write(&path, content).unwrap();
        path
    }

    fn valid_json() -> &'static str {
        r#"{
        "build": { "run": ["echo building", "echo done"] },
        "test":  { "run": ["echo testing"] },
        "empty": { "run": [] }
    }"#
    }

    #[test]
    fn is_json_with_json_extension() {
        assert!(is_json(&PathBuf::from("file.json")));
    }

    #[test]
    fn is_json_with_uppercase_extension() {
        assert!(is_json(&PathBuf::from("file.JSON")));
    }

    #[test]
    fn is_json_with_mixed_case_extension() {
        assert!(is_json(&PathBuf::from("file.Json")));
    }

    #[test]
    fn is_json_with_wrong_extension() {
        assert!(!is_json(&PathBuf::from("file.txt")));
    }

    #[test]
    fn is_json_with_no_extension() {
        assert!(!is_json(&PathBuf::from("file")));
    }

    #[test]
    fn is_json_with_toml_extension() {
        assert!(!is_json(&PathBuf::from("config.toml")));
    }

    #[test]
    fn validate_path_file_does_not_exist() {
        let path = PathBuf::from("/nonexistent/path/file.json");
        assert!(validate_path(&path).is_err());
    }

    #[test]
    fn validate_path_is_a_directory() {
        let dir = TempDir::new().unwrap();
        let result = validate_path(&dir.path().to_path_buf());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is not a file"));
    }

    #[test]
    fn validate_path_not_a_json_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "content").unwrap();
        let result = validate_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a JSON file"));
    }

    #[test]
    fn validate_path_valid_json_file() {
        let dir = TempDir::new().unwrap();
        let path = tmp_json(&dir, "{}");
        assert!(validate_path(&path).is_ok());
    }

    #[test]
    fn validate_path_returns_same_path_on_success() {
        let dir = TempDir::new().unwrap();
        let path = tmp_json(&dir, "{}");
        let result = validate_path(&path).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn parse_valid_scripts_json() {
        let scripts: HashMap<String, Script> = serde_json::from_str(valid_json()).unwrap();
        assert!(scripts.contains_key("build"));
        assert!(scripts.contains_key("test"));
        assert!(scripts.contains_key("empty"));
    }

    #[test]
    fn parse_script_commands_are_correct() {
        let scripts: HashMap<String, Script> = serde_json::from_str(valid_json()).unwrap();
        let build = scripts.get("build").unwrap();
        assert_eq!(build.run.len(), 2);
        assert_eq!(build.run[0], "echo building");
        assert_eq!(build.run[1], "echo done");
    }

    #[test]
    fn parse_empty_run_array() {
        let scripts: HashMap<String, Script> = serde_json::from_str(valid_json()).unwrap();
        assert_eq!(scripts.get("empty").unwrap().run.len(), 0);
    }

    #[test]
    fn parse_invalid_json_returns_error() {
        let result: Result<HashMap<String, Script>, _> = serde_json::from_str("{ not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_wrong_schema_missing_run_field() {
        let result: Result<HashMap<String, Script>, _> =
            serde_json::from_str(r#"{ "build": { "commands": ["echo hi"] } }"#);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_json_object() {
        let scripts: HashMap<String, Script> = serde_json::from_str("{}").unwrap();
        assert!(scripts.is_empty());
    }

    #[test]
    fn parse_script_with_cd_command() {
        let scripts: HashMap<String, Script> =
            serde_json::from_str(r#"{ "setup": { "run": ["cd /tmp", "echo hello"] } }"#).unwrap();
        assert_eq!(scripts.get("setup").unwrap().run[0], "cd /tmp");
    }

    #[test]
    fn cd_detection_starts_with_cd_space() {
        assert!("cd /tmp".starts_with("cd "));
    }

    #[test]
    fn cd_detection_plain_cd_no_space() {
        assert!(!"cd".starts_with("cd "));
    }

    #[test]
    fn cd_detection_other_command() {
        assert!(!"echo cd".starts_with("cd "));
    }

    #[test]
    fn cd_arg_extraction_normal_path() {
        assert_eq!("cd /tmp/folder"[3..].trim(), "/tmp/folder");
    }

    #[test]
    fn cd_arg_extraction_with_spaces() {
        assert_eq!("cd   /tmp/folder  "[3..].trim(), "/tmp/folder");
    }

    #[test]
    fn cd_arg_extraction_empty_gives_empty() {
        assert!("cd "[3..].trim().is_empty());
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

    #[test]
    fn script_lookup_existing_key() {
        let scripts: HashMap<String, Script> = serde_json::from_str(valid_json()).unwrap();
        assert!(scripts.get("build").is_some());
    }

    #[test]
    fn script_lookup_missing_key() {
        let scripts: HashMap<String, Script> = serde_json::from_str(valid_json()).unwrap();
        assert!(scripts.get("nonexistent").is_none());
    }

    #[test]
    fn script_lookup_case_sensitive() {
        let scripts: HashMap<String, Script> = serde_json::from_str(valid_json()).unwrap();
        assert!(scripts.get("Build").is_none());
        assert!(scripts.get("build").is_some());
    }

    #[test]
    fn saved_path_empty_os_str_is_detected() {
        assert!(PathBuf::from("").as_os_str().is_empty());
    }

    #[test]
    fn saved_path_nonempty_os_str_is_detected() {
        assert!(!PathBuf::from("/some/path.json").as_os_str().is_empty());
    }

    #[test]
    fn script_struct_serializes_correctly() {
        let script = Script {
            run: vec!["echo hi".to_string(), "echo bye".to_string()],
        };
        let json = serde_json::to_string(&script).unwrap();
        assert!(json.contains("echo hi"));
        assert!(json.contains("echo bye"));
    }

    #[test]
    fn script_struct_deserializes_correctly() {
        let script: Script = serde_json::from_str(r#"{ "run": ["echo hello"] }"#).unwrap();
        assert_eq!(script.run.len(), 1);
        assert_eq!(script.run[0], "echo hello");
    }
}
