mod ui;
mod events;
mod directory;
mod my_errors;

use std::io;

use crate::ui::App;

fn main() -> io::Result<()>{
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

