use std::fs;
use std::path::Path;

pub fn load_index() -> String {
    if let Some(path) = get_config_path() {
        if let Ok(script) = fs::read_to_string(path.join("init.txt")) {
            return script;
        }
    }
    return String::new();
}

fn get_config_path() -> Option<&'static Path> {
    let bundled = Path::new("config");

    if bundled.exists() {
        return Some(bundled);
    }

    // will add platform specific stuff soon

    let linux_config = Path::new("~/.config/full_calc");

    if linux_config.exists() {
        return Some(linux_config)
    }

    None
}
