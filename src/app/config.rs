use ratatui::style::Color;

use crate::parser::tokens::Token;

use super::state::App;

impl App<'_> {

    // this should probably be implemented on config directly while returning output msg
    pub fn update_config(&mut self, input: &Vec<Token>) {
        match input.get(1) {
            Some(Token::Identifier(config_opt)) => match config_opt.as_str() {
                "trig" => {
                    match input.get(2) {
                        Some(Token::Identifier(trig_opt)) => match trig_opt.as_str() {
                            "deg" => self.config.is_radians = false,
                            "rad" => self.config.is_radians = true,
                            _ => {
                                self.context.push_history_text("trig mode options are 'deg', 'rad'");
                                return;
                            },
                        },
                        _ => self.config.is_radians = !self.config.is_radians,
                    }
                    let opt_text = match self.config.is_radians {
                        true => "rad",
                        false => "deg",
                    };
                    self.context.push_history_text(&format!("trig mode set to {opt_text}"));
                },
                "theme" => match input.get(2) {
                    Some(Token::Identifier(theme_opt)) => {
                        let color_text = match input.get(3) {
                            Some(Token::Identifier(color_text)) => color_text,
                            Some(Token::Number(color_text)) => color_text,
                            _ => {
                                self.context.push_history_text("theme requires color in hex format: 'config theme text AABBCC");
                                return;
                            },
                        };
                        let color = match u32::from_str_radix(color_text, 16) {
                            Ok(color) => color,
                            Err(_) => {
                                self.context.push_history_text("expected hex color");
                                return
                            },
                        };
                        let color = Color::from_u32(color);
                        match theme_opt.as_str() {
                            "number" => self.config.theme.number = color,
                            "identifier" => self.config.theme.identifier = color,
                            "unknownIdentifier" => self.config.theme.unknown_identifier = color,
                            "command" => self.config.theme.command = color,
                            "operator" => self.config.theme.operator = color,
                            "inputBg" => self.config.theme.input_line_bg = color,
                            "resultBg" => self.config.theme.result_line_bg = color,
                            "currentBg" => self.config.theme.current_line_bg = color,
                            "text" => self.config.theme.text = color,
                            _ => self.context.push_history_text("unknown theme option"),
                        }
                    },
                    _ => self.context.push_history_text("use 'show commands' to see all theme options"),
                },
                _ => self.context.push_history_text("use 'show commands' to get a complete list of config options"),
            },
            _ => self.context.push_history_text("use 'show commands' to get a complete list of config options"),
        };
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
