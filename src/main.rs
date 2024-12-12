use std::io;

use full_calc::app::state::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    // terminal.clear()?;
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
