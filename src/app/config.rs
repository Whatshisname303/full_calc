use ratatui::style::Color;

use crate::parser::tokens::Token;

use super::user_scripts::{self, DEFAULT_INIT_SCRIPT_CONTENT};

impl Config {
    pub fn update_from_tokens(&mut self, input: &[Token]) -> String {
        let config_opt = match input.get(0) {
            Some(Token::Identifier(config_opt)) => config_opt.as_str(),
            _ => return "use 'show help' to get a complete list of config options".to_string(),
        };
        match config_opt {
            "script" => self.update_script(&input[1..]),
            "trig" => self.update_trig(&input[1..]),
            "theme" => self.update_theme(&input[1..]),
            _ => format!("unknown option {}, use 'show help' to get a complete list of config options", config_opt),
        }
    }

    fn update_script(&mut self, input: &[Token]) -> String {
        let script_opt = match input.get(0) {
            Some(Token::Identifier(script_opt)) => script_opt.as_str(),
            _ => "",
        };
        let config_path = match user_scripts::guessed_config_path() {
            Ok(config_path) => config_path,
            Err(e) => return e.to_string(),
        };
        match script_opt {
            "create" => {
                if config_path.exists() {
                    return format!("config already exists at {}", config_path.to_string_lossy());
                }

                let err = std::fs::create_dir(&config_path)
                    .and_then(|_| std::fs::write(config_path.join("init.txt"), DEFAULT_INIT_SCRIPT_CONTENT));

                match err {
                    Ok(()) => format!("created config at {}", config_path.to_string_lossy()),
                    Err(e) => e.to_string(),
                }
            },
            "show" => match config_path.exists() {
                true => format!("config exists at {}", config_path.to_string_lossy()),
                false => format!(
                    "no config exists, use 'config script create' or create a folder at {}",
                    config_path.to_string_lossy(),
                ),
            },
            "open" => {
                // TODO
                return "not implemented".to_string();
            },
            _ => "script options: config script <create/show/open>".to_string()
        }
    }

    fn update_trig(&mut self, input: &[Token]) -> String {
        let trig_opt = match input.get(0) {
            Some(Token::Identifier(trig_opt)) => trig_opt.as_str(),
            _ => "",
        };
        match trig_opt {
            "deg" => {
                self.is_radians = false;
                "trig mode set to deg".to_string()
            },
            "rad" => {
                self.is_radians = true;
                "trig mode set to rad".to_string()
            },
            "" => {
                self.is_radians = !self.is_radians;
                format!(
                    "trig mode set to {}",
                    match self.is_radians {
                        true => "rad",
                        false => "deg",
                    }
                )
            }
            _ => "trig mode options are 'deg', 'rad'".to_string(),
        }
    }

    fn update_theme(&mut self, input: &[Token]) -> String {
        let theme_opt = match input.get(0) {
            Some(Token::Identifier(theme_opt)) => theme_opt,
            _ => return "use 'show help' to show all theme options".to_string(),
        };
        let color_text = match input.get(1) {
            Some(Token::Identifier(color_text)) => color_text,
            Some(Token::Number(color_text)) => color_text,
            _ => return "theme requires color in hex format: 'config theme text AABBCC'".to_string(),
        };
        let color = match u32::from_str_radix(color_text, 16) {
            Ok(color) => Color::from_u32(color),
            Err(_) => return "theme requires color in hex format: 'config theme text AABBCC'".to_string(),
        };
        match theme_opt.as_str() {
            "number" => self.theme.number = color,
            "identifier" => self.theme.identifier = color,
            "unknownIdentifier" => self.theme.unknown_identifier = color,
            "command" => self.theme.command = color,
            "operator" => self.theme.operator = color,
            "inputBg" => self.theme.input_line_bg = color,
            "resultBg" => self.theme.result_line_bg = color,
            "currentBg" => self.theme.current_line_bg = color,
            "text" => self.theme.text = color,
            _ => return "unknown theme option".to_string(),
        };
        return "theme option updated".to_string();
    }
}

#[derive(Debug)]
pub struct Theme {
    pub number: Color,
    pub identifier: Color,
    pub unknown_identifier: Color,
    pub command: Color,
    pub operator: Color,
    pub input_line_bg: Color,
    pub result_line_bg: Color,
    pub current_line_bg: Color,
    pub divider: Color,
    pub text: Color,
    pub cursor: Color,

    pub v_show_dividers: bool,
    pub v_cursor: char,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            number: Color::Rgb(230, 134, 57),
            identifier: Color::Rgb(240, 240, 240),
            unknown_identifier: Color::Rgb(180, 180, 180),
            command: Color::Rgb(255, 87, 87),
            operator: Color::Rgb(232, 208, 151),
            input_line_bg: Color::Rgb(60, 60, 60),
            result_line_bg: Color::Rgb(40, 40, 40),
            current_line_bg: Color::Rgb(60, 60, 60),
            divider: Color::White,
            text: Color::Rgb(240, 240, 240),
            cursor: Color::White,
            v_show_dividers: true,
            v_cursor: '|',
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Panel {
    Variables,
    Autocomplete,
    ExpPreview,
}

#[derive(Debug)]
pub struct Config {
    pub show_output: bool,
    pub auto_brace: bool,
    pub expand_matrices: bool,
    pub is_radians: bool,

    pub theme: Theme,
    pub panels: Vec<Panel>,
    pub tab_width: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            show_output: true,
            auto_brace: true,
            expand_matrices: true,
            is_radians: false,
            theme: Theme::default(),
            panels: vec![Panel::Autocomplete, Panel::Variables],
            tab_width: 4,
        }
    }
}
