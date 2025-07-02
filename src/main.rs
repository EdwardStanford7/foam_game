/*
    Modules
*/

mod game;
mod editing;
mod playing;
mod tile;

/*
    Game entrypoint
*/

use game::FoamGame;

use eframe::{self, NativeOptions};
use egui;

fn main() -> Result<(), eframe::Error> {
    let mut options = NativeOptions::default();
    options.viewport.resizable = Some(true);
    options.viewport.inner_size = Some(egui::vec2(1600.0, 900.0));

    eframe::run_native(
        "Foam Game",
        options,
        Box::new(|_cc| Ok(Box::new(FoamGame::default()))),
    )
}
