use std::io;

use full_calc::app::state::App;

// fn main() {
//     println!("Using {} text", full_calc::app::config::load_index());
// }

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    // terminal.clear()?;
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
