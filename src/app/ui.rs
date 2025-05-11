use std::iter;

use layout::Flex;
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};
use symbols::border;

use crate::parser::highlighting::{get_highlight_tokens, HighlightToken, HighlightTokenType};

use super::{config::Panel, state::{App, PopupName}};

impl App<'_> {
    fn map_token_colors(&self, token: &HighlightToken) -> Span<'_> {
        let theme = &self.config.theme;
        match token.kind {
            HighlightTokenType::Identifier => {
                match self.context.get_var(&token.text).is_some() || self.context.get_function(&token.text).is_some() {
                    true => token.text.clone().fg(theme.identifier),
                    false => token.text.clone().fg(theme.unknown_identifier),
                }
            },
            HighlightTokenType::Number => token.text.clone().fg(theme.number),
            HighlightTokenType::Operator => token.text.clone().fg(theme.operator),
            HighlightTokenType::Command => token.text.clone().fg(theme.command),
            HighlightTokenType::Space => token.text.clone().fg(Color::Black),
            HighlightTokenType::Tab => " ".repeat(self.config.tab_width).fg(Color::Black),
            HighlightTokenType::Newline => panic!("newlines are delimiters"),
        }
    }

    pub fn get_horizontal_layout(&self, area: Rect) -> (Rect, Rect) {
        let [text_area, panel_area] = match self.config.panels.len() {
            0 => Layout::horizontal([
                Constraint::Percentage(100),
                Constraint::Percentage(0),
            ]),
            _ => Layout::horizontal([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ]),
        }.areas(area);
        (text_area, panel_area)
    }

    fn render_text_area(&self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        for entry in &self.context.history {
            lines.push(Vec::new());

            let mut line_length = 0;

            let bg_color = match entry.is_output {
                true => self.config.theme.result_line_bg,
                false => self.config.theme.input_line_bg,
            };

            for token in &entry.tokens {
                let token_len = token.text.len() as u16;
                line_length += token_len;
                if token.kind == HighlightTokenType::Newline {
                    lines.push(Vec::new());
                    line_length = 0;
                    continue;
                }
                if line_length > area.width && token.kind != HighlightTokenType::Space {
                    if line_length - token_len > 0 {
                        lines.push(Vec::new());
                        line_length = token_len;
                    }
                }
                let span = self.map_token_colors(token).bg(bg_color);
                lines.last_mut().unwrap().push(span);
            }
        }

        let mut lines: Vec<_> = lines.into_iter().map(Line::from).collect();

        let mut current_line = get_highlight_tokens(&self.context.current_line)
            .iter()
            .map(|token| self.map_token_colors(token))
            .collect::<Vec<_>>();

        current_line.push(Span::from(&self.config.cursor).fg(self.config.theme.cursor));

        lines.push(Line::from(current_line).bg(self.config.theme.current_line_bg));

        Paragraph::new(lines).scroll((self.context.history_scroll, 0)).render(area, buf);
    }

    fn render_panels(&self, area: Rect, buf: &mut Buffer) {
        let panel_count = self.config.panels.len();
        let constraints = iter::repeat(Constraint::Percentage((100 / panel_count) as u16)).take(panel_count);
        let panel_layout = Layout::vertical(constraints).split(area);

        let panels: Vec<_> = self.config.panels.iter()
            .map(|panel| match panel {
                Panel::Variables => self.get_vars_panel(),
                Panel::Autocomplete => self.get_autocomplete_panel(),
                Panel::ExpPreview => self.get_preview_panel(),
            })
            .collect();

        for (i, panel) in panels.iter().enumerate() {
            panel.render(panel_layout[i], buf);
        }
    }

    fn get_vars_panel(&self) -> Paragraph<'_> {
        let block = Block::bordered().title(Line::from("Vars".bold())).border_set(border::THICK);
        let vars: Vec<_> = self.context.vars.iter()
            .map(|(name, value)| Line::from(format!("{} = {}", name, value.short_string())))
            .rev()
            .collect();
        Paragraph::new(Text::from(vars)).block(block)
    }

    fn get_autocomplete_panel(&self) -> Paragraph<'_> {
        let lines = match get_highlight_tokens(&self.context.current_line).last() {
            Some(token) => match token.kind {
                HighlightTokenType::Identifier => self.context.vars.iter()
                    .map(|(name, value)| (name, Some(value)))
                    .chain(self.context.functions.iter().map(|function_def| (&function_def.name, None)))
                    .filter(|(name, _)| name.contains(&token.text))
                    .map(|(name, value)| {
                        let (left_side, right_side) = name.split_once(&token.text).unwrap();
                        let mut line_tokens = vec![
                            left_side.fg(self.config.theme.unknown_identifier),
                            token.text.clone().fg(self.config.theme.identifier),
                            right_side.fg(self.config.theme.unknown_identifier),
                        ];
                        if let Some(value) = value {
                            line_tokens.push(" = ".fg(self.config.theme.operator));
                            line_tokens.push(value.short_string().fg(self.config.theme.number));
                        }
                        Line::from(line_tokens)
                    })
                    .collect(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let block = Block::bordered().title(Line::from("Autocomplete".bold())).border_set(border::THICK);
        Paragraph::new(Text::from(lines)).block(block)
    }

    fn get_preview_panel(&self) -> Paragraph<'_> {
        // TODO
        let block = Block::bordered().title(Line::from("Preview".bold())).border_set(border::THICK);
        Paragraph::new(Text::from("Hello")).block(block)
    }

    fn get_vars_popup(&self) -> Paragraph<'_> {
        let lines: Vec<_> = self.context.vars.iter()
            .map(|(name, value)| Line::from(format!("{} = {:?}", name, value)))
            .rev()
            .collect();
        let block = Block::bordered().title("Vars");
        Paragraph::new(lines).scroll((self.context.modal_scroll, 0)).block(block)
    }

    fn get_functions_popup(&self) -> Paragraph<'_> {
        let lines: Vec<_> = self.context.functions.iter()
            .map(|function_def| Line::from(format!("{}({})", function_def.name, function_def.params.join(", "))))
            .rev()
            .collect();
        let block = Block::bordered().title("Functions");
        Paragraph::new(lines).scroll((self.context.modal_scroll, 0)).block(block)
    }

    fn get_help_popup(&self) -> Paragraph<'_> {
        let lines = [
            "General:",
            "      More detailed docs available at github.com/Whatshisname303/full_calc",
            "",
            "      Define matrices by separating columns with ',' and rows with ';'",
            "      ex: [1, 2, 3; 4, 5, 6; 7, 8, 9]",
            "",
            "      Vectors are just matrices with a single column",
            "",
            "Controls:",
            "    - arrowkeys: scroll view up and down",
            "    - ctrl + up: copy line",
            "    - ctrl + backspace: delete line",
            "    - q: close active modal",
            "    - tab: autocomplete",
            "",
            "Commands:",
            "    - clear: clears output",
            "    - quit or exit: exit the program",
            "    - reload: reloads entire context (output, variables, etc)",
            "    - use <namespace>: brings namespaced variables into global scope",
            "    - load <script>: executes the given script",
            "    - def: used to define functions",
            "    - config <option>: used to edit config values",
            "    - show <vars/functions/help>: used to show modals like this",
            "    - panel <vars/autocomplete>: toggles a panel",
            "",
            "Config Options:",
            "      Really there isn't a ton to configure and I never bothered to",
            "      make the UI look any good. The docs should have detailed",
            "      explanations of everything you can mess with, although this can",
            "      be a quick reference to check the options.",
            "",
            "    - script",
            "       - show (shows the current config directory if it exists)",
            "       - create (creates a config directory at the default location for your system)",
            "    - theme (change your colors for current session)",
            "       - number <color>",
            "       - identifier <color>",
            "       - unknownIdentifier <color>",
            "       - command <color>",
            "       - operator <color>",
            "       - inputBg <color>",
            "       - resultBg <color>",
            "       - currentBg <color>",
            "       - text <color>",
        ];
        let lines: Vec<_> = lines.iter()
            .map(|line| Line::from(line.to_string()))
            .collect();
        let block = Block::bordered().title("Help");
        Paragraph::new(lines).scroll((self.context.modal_scroll, 0)).block(block)
    }

    fn render_popup(&self, popup: &PopupName, area: Rect, buf: &mut Buffer) {
        let [popup_area] = Layout::horizontal([Constraint::Percentage(90)])
            .flex(Flex::Center)
            .areas(area);
        let [popup_area] = Layout::vertical([Constraint::Percentage(90)])
            .flex(Flex::Center)
            .areas(popup_area);

        Clear.render(popup_area, buf);

        let popup_context = match popup {
            PopupName::Vars => self.get_vars_popup(),
            PopupName::Functions => self.get_functions_popup(),
            PopupName::Help => self.get_help_popup(),
        };

        popup_context.render(popup_area, buf);
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (text_area, panel_area) = self.get_horizontal_layout(area);

        self.render_text_area(text_area, buf);

        if self.config.panels.len() > 0 {
            self.render_panels(panel_area, buf);
        }

        if let Some(popup) = &self.context.current_popup {
            self.render_popup(popup, area, buf);
        }
    }
}
