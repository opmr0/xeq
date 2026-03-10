use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::types::{SavedPath, Scripts};

pub fn validate_path(path: &PathBuf) -> Result<PathBuf, io::Error> {
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
    } else if !is_toml(path) {
        return Err(io::Error::other(format!("{:?} is not a TOML file", path)));
    }
    Ok(path.clone())
}

pub fn is_toml(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("toml"))
        .unwrap_or(false)
}

pub fn save_path(path: PathBuf) -> Result<(), io::Error> {
    validate_path(&path)?;
    let config = SavedPath {
        path: path.canonicalize().unwrap(),
    };
    confy::store("xeq", "path", &config)
        .map_err(|e| io::Error::other(format!("Failed to store config: {}", e)))
}

pub fn load_path() -> Option<PathBuf> {
    let config: SavedPath = confy::load("xeq", "path").ok()?;
    if config.path.as_os_str().is_empty() {
        return None;
    }
    Some(config.path)
}

pub fn read_scripts(global: bool) -> Result<Scripts, std::io::Error> {
    let file_path = PathBuf::from("./xeq.toml");

    let path = if global {
        load_path().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "xeq is not configured. Run: xeq config <path/to/file.toml>",
            )
        })?
    } else if file_path.exists() {
        file_path
    } else {
        load_path().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "xeq is not configured. Run: xeq config <path/to/file.toml>",
            )
        })?
    };

    let content = fs::read_to_string(path)?;
    toml::from_str::<Scripts>(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp_toml(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("xeq.toml");
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn is_toml_with_toml_extension() {
        assert!(is_toml(&PathBuf::from("file.toml")));
    }

    #[test]
    fn is_toml_with_uppercase_extension() {
        assert!(is_toml(&PathBuf::from("file.TOML")));
    }

    #[test]
    fn is_toml_with_mixed_case_extension() {
        assert!(is_toml(&PathBuf::from("file.Toml")));
    }

    #[test]
    fn is_toml_with_wrong_extension() {
        assert!(!is_toml(&PathBuf::from("file.txt")));
    }

    #[test]
    fn is_toml_with_no_extension() {
        assert!(!is_toml(&PathBuf::from("file")));
    }

    #[test]
    fn is_toml_with_json_extension() {
        assert!(!is_toml(&PathBuf::from("config.json")));
    }

    #[test]
    fn validate_path_file_does_not_exist() {
        let path = PathBuf::from("/nonexistent/path/file.toml");
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
    fn validate_path_not_a_toml_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, "content").unwrap();
        let result = validate_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a TOML file"));
    }

    #[test]
    fn validate_path_valid_toml_file() {
        let dir = TempDir::new().unwrap();
        let path = tmp_toml(&dir, "");
        assert!(validate_path(&path).is_ok());
    }

    #[test]
    fn validate_path_returns_same_path_on_success() {
        let dir = TempDir::new().unwrap();
        let path = tmp_toml(&dir, "");
        let result = validate_path(&path).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn saved_path_empty_os_str_is_detected() {
        assert!(PathBuf::from("").as_os_str().is_empty());
    }

    #[test]
    fn saved_path_nonempty_os_str_is_detected() {
        assert!(!PathBuf::from("/some/path.toml").as_os_str().is_empty());
    }
}
