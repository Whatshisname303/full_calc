use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub show_output: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            show_output: true,
        }
    }
}

pub fn load_index() -> String {
    if let Some(path) = get_config_path() {
        if let Ok(script) = fs::read_to_string(path.join("init.txt")) {
            return script;
        }
    }
    return String::new();
}

// pub fn get_config() -> Config {
//     let config = match get_config_path() {
//         Some(path) => {

//         },
//         None => {},
//     };

//     Config {}
// }

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
