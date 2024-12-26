use ratatui::style::Color;

#[derive(Debug)]
pub struct Theme {
    pub number: Color,
    pub identifier: Color,
    pub unknown_identifier: Color,
    pub function: Color,
    pub command: Color,
    pub operator: Color,
    pub line_background: Color,
    pub result_background: Color,
    pub panel_background: Color,
    pub divider: Color,
    pub text: Color,
    pub cursor: Color,

    pub v_show_dividers: bool,
    pub v_cursor: char,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            number: Color::Green,
            identifier: Color::LightGreen,
            unknown_identifier: Color::Cyan,
            function: Color::LightMagenta,
            command: Color::Red,
            operator: Color::LightYellow,
            line_background: Color::Gray,
            result_background: Color::Black,
            panel_background: Color::DarkGray,
            divider: Color::White,
            text: Color::White,
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
    pub panels: Vec<Panel>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            show_output: true,
            auto_brace: true,
            expand_matrices: true,
            is_radians: false,
            theme: Theme::default(),
            panels: vec![Panel::ExpPreview, Panel::Autocomplete, Panel::Variables],
        }
    }
}
