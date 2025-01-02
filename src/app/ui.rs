use std::iter;

use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use symbols::border;

use crate::parser::highlighting::{get_highlight_tokens, HighlightToken, HighlightTokenType};

use super::{config::Panel, state::App};

impl App {
    fn get_horizontal_layout(&self, area: Rect) -> (Rect, Rect) {
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
        let theme = &self.config.theme;
        let map_token_colors = |token: &HighlightToken| match token.kind {
            HighlightTokenType::Identifier => match self.context.get_var(&token.text) {
                Some(_) => token.text.clone().fg(theme.identifier),
                None => token.text.clone().fg(theme.unknown_identifier),
            },
            HighlightTokenType::Number => token.text.clone().fg(theme.number),
            HighlightTokenType::Operator => token.text.clone().fg(theme.operator),
            HighlightTokenType::Command => token.text.clone().fg(theme.command),
            HighlightTokenType::Space => token.text.clone().fg(Color::Black),
            HighlightTokenType::Tab => " ".repeat(self.config.tab_width).fg(Color::Black),
            HighlightTokenType::Newline => panic!("newlines are delimiters"),
        };

        let mut lines: Vec<Line<'_>> = self.context.history.iter().flat_map(|history_entry| {
            let bg_color = match history_entry.is_output {
                true => theme.result_line_bg,
                false => theme.input_line_bg,
            };
            history_entry.tokens
                .split(|token| token.kind == HighlightTokenType::Newline)
                .map(|tokens|
                    tokens.iter()
                        .map(map_token_colors)
                        .collect::<Vec<_>>()
                )
                .map(|tokens| Line::from(tokens).bg(bg_color))
                .collect::<Vec<_>>()
        }).collect();

        let current_line = get_highlight_tokens(&self.context.current_line)
            .iter()
            .map(map_token_colors)
            .collect::<Vec<_>>();

        lines.push(Line::from(current_line).bg(theme.current_line_bg));

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
        // TODO
        let block = Block::bordered().title(Line::from("Autocomplete".bold())).border_set(border::THICK);
        Paragraph::new(Text::from("Hi")).block(block)
    }

    fn get_preview_panel(&self) -> Paragraph<'_> {
        // TODO
        let block = Block::bordered().title(Line::from("Preview".bold())).border_set(border::THICK);
        Paragraph::new(Text::from("Hello")).block(block)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (text_area, panel_area) = self.get_horizontal_layout(area);

        self.render_text_area(text_area, buf);

        if self.config.panels.len() > 0 {
            self.render_panels(panel_area, buf);
        }
    }
}
