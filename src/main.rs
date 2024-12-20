use std::io;

use full_calc::app::state::App;

// fn main() {
//     match full_calc::app::user_scripts::read_script("init.txt") {
//         Ok(script) => {dbg!(script);},
//         Err(e) => {dbg!(e);},
//     };
// }

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    // terminal.clear()?;
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
