use ratatui::{
    prelude::*,
    style::Stylize,
};

use super::state::App;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let title = Line::from(" Counter App Tutorial ".bold());
        // let instructions = Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q> ".blue().bold(),
        // ]);
        // let block = Block::bordered()
        //     .title(title.centered())
        //     .title_bottom(instructions.centered())
        //     .border_set(border::THICK);

        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);

        // Paragraph::new(counter_text)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);


        // let input = Text::from(self.line.as_str());
        // let previous_lines = Paragraph::new(self.history.iter().map(|line| Text::from(line.as_str())).collect::<Vec<_>>());
        let mut previous_lines = Text::from(self.history.iter().map(|l| Line::from(l.as_str())).collect::<Vec<_>>());


        let current_line = Line::from(
            vec![
                self.current_line.as_str().into(),
                "|".into(),
            ]
        ).bg(Color::Black);

        previous_lines.push_line(current_line);

        previous_lines.render(area, buf);
        // current_line.render(area, buf);
        // Line::from(self.line.as_str()).render(area, buf);


    }
}
