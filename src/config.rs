use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::types::{Config, SavedPath};

pub fn validate_path(path: &PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        bail!("{:?} does not exist", path);
    }
    if path.is_dir() {
        bail!("{:?} is a directory, not a file", path);
    }
    if !is_toml(path) {
        bail!("{:?} is not a TOML file", path);
    }
    Ok(path.clone())
}

pub fn is_toml(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("toml"))
        .unwrap_or(false)
}

pub fn save_path(path: PathBuf) -> Result<()> {
    validate_path(&path)?;
    let config = SavedPath {
        path: path.canonicalize().context("failed to canonicalize path")?,
    };
    confy::store("xeq", "path", &config).context("failed to save config")?;
    Ok(())
}

pub fn load_path() -> Option<PathBuf> {
    let config: SavedPath = confy::load("xeq", "path").ok()?;
    if config.path.as_os_str().is_empty() {
        return None;
    }
    Some(config.path)
}

pub fn read_scripts(global: bool) -> Result<Config> {
    let local = PathBuf::from("./xeq.toml");

    let path = if global {
        load_path().context("xeq is not configured. Run: xeq config <path/to/file.toml>")?
    } else if local.exists() {
        local
    } else {
        load_path().context("xeq is not configured. Run: xeq config <path/to/file.toml>")?
    };

    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    toml::from_str::<Config>(&content)
        .with_context(|| format!("failed to parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_toml(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("xeq.toml");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn is_toml_accepts_toml_extension() {
        assert!(is_toml(Path::new("file.toml")));
    }

    #[test]
    fn is_toml_is_case_insensitive() {
        assert!(is_toml(Path::new("file.TOML")));
        assert!(is_toml(Path::new("file.Toml")));
    }

    #[test]
    fn is_toml_rejects_other_extensions() {
        assert!(!is_toml(Path::new("file.txt")));
        assert!(!is_toml(Path::new("file.json")));
        assert!(!is_toml(Path::new("file")));
    }

    #[test]
    fn validate_path_rejects_missing_file() {
        let result = validate_path(&PathBuf::from("/nonexistent/path/file.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn validate_path_rejects_directory() {
        let dir = TempDir::new().unwrap();
        let result = validate_path(&dir.path().to_path_buf());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("directory"));
    }

    #[test]
    fn validate_path_rejects_non_toml_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, "{}").unwrap();
        let result = validate_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a TOML"));
    }

    #[test]
    fn validate_path_accepts_valid_toml_file() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, "");
        assert!(validate_path(&path).is_ok());
    }

    #[test]
    fn validate_path_returns_the_same_path() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, "");
        assert_eq!(validate_path(&path).unwrap(), path);
    }

    #[test]
    fn read_scripts_parses_valid_toml() {
        let dir = TempDir::new().unwrap();
        write_toml(
            &dir,
            r#"
[build]
run = ["cargo build"]
"#,
        );
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let result = read_scripts(false);
        std::env::set_current_dir(original).unwrap();
        assert!(result.is_ok());
        assert!(result.unwrap().scripts.contains_key("build"));
    }

    #[test]
    fn read_scripts_errors_on_invalid_toml() {
        let dir = TempDir::new().unwrap();
        write_toml(&dir, "{ this is not valid toml");
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let result = read_scripts(false);
        std::env::set_current_dir(original).unwrap();
        assert!(result.is_err());
    }
}
