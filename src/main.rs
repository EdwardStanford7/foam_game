/*
    Modules
*/

mod editing_model;
mod game_ui;
mod item;
mod playing_model;
mod tile;

/*
    Game entrypoint
*/

use eframe::{self, NativeOptions};
use game_ui::App;

fn main() -> Result<(), eframe::Error> {
    let mut options = NativeOptions::default();
    options.viewport.resizable = Some(true);
    options.viewport.inner_size = Some(egui::vec2(1600.0, 900.0));

    eframe::run_native(
        "Foam Game",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
