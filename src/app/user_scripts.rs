use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum ScriptError {
    NoConfigPath,
    ScriptNotFound(String),
}

pub fn read_script(name: &str) -> Result<String, ScriptError> {
    let config_path = get_config_path()?;

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

fn get_config_path() -> Result<&'static Path, ScriptError> {
    let bundled = Path::new("config");

    if bundled.exists() {
        return Ok(bundled);
    }

    // will add platform specific stuff soon

    let linux_config = Path::new("~/.config/full_calc");

    if linux_config.exists() {
        return Ok(linux_config)
    }

    Err(ScriptError::NoConfigPath)
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::NoConfigPath => write!(f, "no config path found"),
            ScriptError::ScriptNotFound(name) => write!(f, "script '{}' not found", name),
        }
    }
}

impl std::error::Error for ScriptError {}
