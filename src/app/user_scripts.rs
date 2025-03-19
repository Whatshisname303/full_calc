use std::fmt;
use std::fs;
use std::path::PathBuf;

pub static DEFAULT_INIT_SCRIPT_CONTENT: &str = "
-- this is a comment from an auto generated script
-- use 'config script show' to see the file location
";

#[derive(Debug)]
pub enum ScriptError {
    NoConfigPath,
    ScriptNotFound(String),
    OsDoesNotSupportConfigDir,
}

pub fn read_script(name: &str) -> Result<String, ScriptError> {
    let config_path = get_config_dir()?;

    let mut script_path = config_path.join(name);

    if let Ok(script_contents) = fs::read_to_string(script_path) {
        return Ok(script_contents);
    }

    script_path = config_path.join(name.to_string() + ".txt");

    if let Ok(script_contents) = fs::read_to_string(script_path) {
        return Ok(script_contents);
    }

    Err(ScriptError::ScriptNotFound(name.to_string()))
}

// this version will return a path if supported by os
pub fn guessed_config_path() -> Result<PathBuf, ScriptError> {
    dirs::config_dir()
        .map(|path| path.join("lcalc".to_string()))
        .ok_or(ScriptError::OsDoesNotSupportConfigDir)
}

// this will return a path if config folder exists
fn get_config_dir() -> Result<PathBuf, ScriptError> {
    let bundled = PathBuf::from("config");

    if bundled.exists() {
        return Ok(bundled);
    }

    let config_path = guessed_config_path()?;

    if config_path.exists() {
        return Ok(config_path)
    }

    Err(ScriptError::NoConfigPath)
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::NoConfigPath => write!(f, "no config path found"),
            ScriptError::ScriptNotFound(name) => write!(f, "script '{}' not found", name),
            ScriptError::OsDoesNotSupportConfigDir => write!(f, "external config only supported on windows/mac/linux"),
        }
    }
}

impl std::error::Error for ScriptError {}
