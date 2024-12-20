use std::error::Error;
use std::fs;
use std::path::Path;

pub fn read_script(name: &str) -> Result<String, Box<dyn Error>> {
    let config_path = get_config_path()?;

    let mut script_path = config_path.join(name);

    if let Ok(script_contents) = fs::read_to_string(script_path) {
        return Ok(script_contents);
    }

    script_path = config_path.join(name.to_string() + ".txt");

    if let Ok(script_contents) = fs::read_to_string(script_path) {
        return Ok(script_contents);
    }

    Err("script not found".into())
}

fn get_config_path() -> Result<&'static Path, Box<dyn Error>> {
    let bundled = Path::new("config");

    if bundled.exists() {
        return Ok(bundled);
    }

    // will add platform specific stuff soon

    let linux_config = Path::new("~/.config/full_calc");

    if linux_config.exists() {
        return Ok(linux_config)
    }

    Err("no config path exists".into())
}
