//!
//! Main game engine using egui
//!
//! Includes three modes: startup mode, editing mode, and playing mode.

use eframe::egui;
use native_dialog::FileDialog;
use std::collections::HashMap;

use super::tile::{ALL_TILES, Tile};
use super::editing::EditingBoard;
use super::playing::PlayingBoard;

#[derive(Debug, Clone)]
pub enum GameMode {
    Startup,
    Editing,
    Playing,
}

pub struct FoamGame {
    mode: GameMode,
    board_size: (usize, usize), // Size of the board as (width, height)
    editing_board: EditingBoard,
    playing_board: Option<PlayingBoard>,
    texture_cache: HashMap<String, egui::TextureHandle>, // Cache for textures to avoid reloading them every frame
    recent_keys: Vec<egui::Key>,
}

impl FoamGame {
    // Add method to get cached texture
    fn get_texture(
        &mut self,
        ctx: &egui::Context,
        tile: &Tile,
    ) -> Result<&egui::TextureHandle, String> {
        let file_name = tile.file_name();

        if !self.texture_cache.contains_key(file_name) {
            // Load and cache the texture
            let image = image::ImageReader::open(file_name)
                .map_err(|err| format!("Error loading texture file at {}: {}", file_name, err))?
                .decode()
                .map_err(|err| format!("Error decoding image at {}: {}", file_name, err))?;

            // Resize the image to 32x32
            let image = image.resize(32, 32, image::imageops::FilterType::Nearest);
            let size = [32, 32]; // Fixed size
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            let texture = ctx.load_texture(
                file_name,
                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                egui::TextureOptions::default(),
            );

            self.texture_cache.insert(file_name.to_string(), texture);
        }

        Ok(self.texture_cache.get(file_name).unwrap())
    }

    fn save_board(&self) -> Result<(), String> {
        // open file dialog to save the board using native_dialog
        let file_path = FileDialog::new()
            .add_filter("Foam Game Board", &["fgb"])
            .set_title("Save Board")
            .show_save_single_file()
            .ok()
            .flatten()
            .ok_or("No file selected".to_string())?;

        // Serialize the board to a file
        let board_data =
            serde_json::to_string(&self.editing_board)
                .map_err(|err| format!("Error serializing board data: {}", err))?;

        std::fs::write(file_path, board_data)
            .map_err(|err| format!("Error writing board file: {}", err))?;

        Ok(())
    }

    fn load_board(&mut self) -> Result<(), String> {
        // open file dialog to load the board using native_dialog
        let file_path = FileDialog::new()
            .add_filter("Foam Game Board", &["fgb"])
            .set_title("Load Board")
            .show_open_single_file()
            .ok()
            .flatten()
            .ok_or("No file selected".to_string())?;

        // Read the board data from the file
        let board_data = std::fs::read_to_string(file_path)
            .map_err(|err| format!("Error reading board file: {}", err))?;

        // Deserialize the board data
        let board: EditingBoard = serde_json::from_str(&board_data)
            .map_err(|err| format!("Error deserializing board data: {}", err))?;

        self.editing_board = board;

        Ok(())
    }

}

impl Default for FoamGame {
    fn default() -> Self {
        Self {
            mode: GameMode::Startup,
            board_size: (0, 0),
            editing_board: Default::default(),
            playing_board: None,
            texture_cache: HashMap::new(),
            recent_keys: Vec::new(),
        }
    }
}

impl eframe::App for FoamGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            GameMode::Startup => startup_screen(ui, self),
            GameMode::Editing => editing_screen(ui, self),
            GameMode::Playing => play_screen(ui, self),
        });
    }
}

/*
    Startup mode
*/

fn startup_screen(ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.heading("Welcome to Foam Game!");

    // Board size selection
    ui.label("Select board size:");

    ui.horizontal(|ui| {
        // MARK: switch to rows columns
        ui.label("Width:");
        ui.add(egui::Slider::new(&mut game.board_size.0, 5..=40).integer());
    });

    ui.horizontal(|ui| {
        ui.label("Height:");
        ui.add(egui::Slider::new(&mut game.board_size.1, 5..=20).integer());
    });

    if ui.button("Start Editing").clicked() {
        // Initialize the board with the selected size
        game.editing_board.set_size(game.board_size.0, game.board_size.1);
        // Switch to editing mode
        game.mode = GameMode::Editing;
    }

    if ui.button("Load Board").clicked() {
        // Load board from file
        if let Err(err) = game.load_board() {
            ui.label(format!("Error loading board: {}", err));
        } else {
            // Switch to editing mode
            game.mode = GameMode::Editing;
        }
    }
}


/*
    Editing mode
*/

fn editing_screen(ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Editing Mode");
    display_editing_menu(ui, game);
    ui.add_space(25.0);
    display_editing_board(ui, game);

    // Keyboard input for tile modification.
    if game.editing_board.has_selected_tile() {
        editing_keyboard_input(ui, game);
    } // Else: no tile selected, skip input handling
}

fn editing_keyboard_input(ui: &mut egui::Ui, game: &mut FoamGame) {
    match game.editing_board.get_selected_tile_mut().expect("Selected tile invalid or not set") {
        Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                directions.up = !directions.up;
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                directions.right = !directions.right;
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                directions.down = !directions.down;
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                directions.left = !directions.left;
            }
        }
        Tile::MoveDiagonal(directions) => {
            let input = ui.input(|i| i.clone());

            for key in [
                egui::Key::ArrowUp,
                egui::Key::ArrowRight,
                egui::Key::ArrowDown,
                egui::Key::ArrowLeft,
            ] {
                if input.key_pressed(key) {
                    game.recent_keys.push(key);
                    if game.recent_keys.len() > 2 {
                        game.recent_keys.remove(0); // Keep only last two
                    }
                }
            }

            if game.recent_keys.len() == 2 {
                use egui::Key::*;
                let (a, b) = (game.recent_keys[0], game.recent_keys[1]);

                match (a, b) {
                    (ArrowUp, ArrowRight) | (ArrowRight, ArrowUp) => {
                        directions.up_right = !directions.up_right;
                        game.recent_keys.clear();
                    }
                    (ArrowDown, ArrowRight) | (ArrowRight, ArrowDown) => {
                        directions.down_right = !directions.down_right;
                        game.recent_keys.clear();
                    }
                    (ArrowDown, ArrowLeft) | (ArrowLeft, ArrowDown) => {
                        directions.down_left = !directions.down_left;
                        game.recent_keys.clear();
                    }
                    (ArrowUp, ArrowLeft) | (ArrowLeft, ArrowUp) => {
                        directions.up_left = !directions.up_left;
                        game.recent_keys.clear();
                    }
                    _ => {}
                }
            }
        }
        Tile::Bounce(val) => {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) && *val < 1) {
                *val += 1; // Increase bounce value
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) && *val > -1) {
                *val -= 1; // Decrease bounce value
            }
        }
        Tile::Portal(c) => {
            ui.input(|i| {
                if i.key_pressed(egui::Key::A) {
                    *c = 'A';
                } else if i.key_pressed(egui::Key::B) {
                    *c = 'B';
                } else if i.key_pressed(egui::Key::C) {
                    *c = 'C';
                } else if i.key_pressed(egui::Key::D) {
                    *c = 'D';
                } else if i.key_pressed(egui::Key::E) {
                    *c = 'E';
                } else if i.key_pressed(egui::Key::F) {
                    *c = 'F';
                } else if i.key_pressed(egui::Key::G) {
                    *c = 'G';
                } else if i.key_pressed(egui::Key::H) {
                    *c = 'H';
                }
            });
        }
        _ => {}
    }
}

fn display_editing_board(ui: &mut egui::Ui, game: &mut FoamGame) {
    // Create a container for modifications
    let mut modifications = Vec::new();

    // Display the board
    egui::Grid::new("editing_board")
        .spacing([1.0, 1.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            for row in 0..game.board_size.1 {
                // in game.board.iter().enumerate() {
                for col in 0..game.board_size.0 {
                    let tile = game.editing_board.get_tile(row, col).unwrap().clone();
                    let response =
                        draw_tile(game, &tile, ui, false);
                    if response.clicked() {
                        // Update the selected tile
                        game.editing_board.select_tile_position(row, col);
                        let selected_type = game.editing_board.get_selected_type().clone();

                        // TODO: this is weird??
                        if std::mem::discriminant(&selected_type)
                            == std::mem::discriminant(game.editing_board.get_tile(row, col).unwrap())
                        {
                            continue; // Skip modification for this tile
                        }

                        // Collect modification for later application
                        // ???
                        // Handle unique tiles (StartSpace and EndSpace)
                        if matches!(selected_type, Tile::StartSpace) {
                            let pos = game.editing_board.get_start_pos();
                            if let Some(pos) = pos {
                                modifications.push((pos.0, pos.1, Tile::Empty));
                            }
                            game.editing_board.set_start_pos(row, col);
                        } else if matches!(selected_type, Tile::EndSpace) {
                            let pos = game.editing_board.get_end_pos();
                            if let Some(pos) = pos {
                                modifications.push((pos.0, pos.1, Tile::Empty));
                            }
                            game.editing_board.set_end_pos(row, col);
                        }

                        modifications.push((row, col, selected_type));
                    }
                    // Draw faint white border around each cell
                    let rect = response.rect;
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_white_alpha(64)),
                        egui::StrokeKind::Outside,
                    );
                }
                ui.end_row();
            }
        });

    // Apply all modifications after iteration is complete
    for (row_idx, col_idx, tile) in modifications {
        *game.editing_board.get_tile_mut(row_idx, col_idx).unwrap() = tile;
    }
}


fn display_editing_menu(ui: &mut egui::Ui, game: &mut FoamGame) {
    // Display menus and buttons for editing the board
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI buttons to change modes and save/load the board
            if ui.button("Switch to Playing Mode").clicked() && game.editing_board.is_playable_board() {
                game.mode = GameMode::Playing;
                game.playing_board = Some(PlayingBoard::new(&game.editing_board).unwrap());
            }
            if ui.button("Save Board").clicked() {
                if let Err(err) = game.save_board() {
                    ui.label(format!("Error saving board: {}", err));
                } else {
                    ui.label("Board saved successfully!");
                }
            }
            if ui.button("Load Board").clicked() {
                if let Err(err) = game.load_board() {
                    ui.label(format!("Error loading board: {}", err));
                } else {
                    ui.label("Board loaded successfully!");
                }
            }

            let selected_type = game.editing_board.get_selected_type().clone();
            ui.label("Selected Tile:").on_hover_text(selected_type.explanation());

            draw_tile(game, &selected_type, ui, false);
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            for tile in ALL_TILES {
                if draw_tile(game, tile, ui, false).clicked() {
                    game.editing_board.select_type(tile.clone());
                }
            }
        });
    });
}

/*
    Play mode
*/

fn play_screen(ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Playing Mode");

    display_playing_board(ui, game);

    handle_player_movement(ui, game);
}

fn handle_player_movement(ui: &mut egui::Ui, board: &mut PlayingBoard, recent_keys: &mut Vec<egui::Key>) {
    // If moving from a cloud tile, remove it
    if matches!(
        board.previous_tile(),
        Tile::Cloud(_)
    ) && board.position_is_new()
    {
        board.set_previous_tile(Tile::Empty);
    }

    let current_tile = board.current_tile();
    let mut new_pos: (isize, isize) = board.get_player_position_isize();

    match current_tile {
        Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
            if ui.input(|i| i.key_down(egui::Key::Space)) {
                // Handle cardinal movement based on allowed directions
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) && directions.up) {
                    new_pos.0 -= 2; // Move up
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) && directions.down) {
                    new_pos.0 += 2; // Move down
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) && directions.left) {
                    new_pos.1 -= 2; // Move left
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) && directions.right) {
                    new_pos.1 += 2; // Move right
                }
            } else {
                // Handle cardinal movement based on allowed directions
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp) && directions.up) {
                    new_pos.0 -= 1; // Move up
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown) && directions.down) {
                    new_pos.0 += 1; // Move down
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft) && directions.left) {
                    new_pos.1 -= 1; // Move left
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight) && directions.right) {
                    new_pos.1 += 1; // Move right
                }
            }
        }
        Tile::MoveDiagonal(directions) => {
            // Handle diagonal movement based on allowed directions
            let input = ui.input(|i| i.clone());

            for key in [
                egui::Key::ArrowUp,
                egui::Key::ArrowRight,
                egui::Key::ArrowDown,
                egui::Key::ArrowLeft,
            ] {
                if input.key_pressed(key) {
                    recent_keys.push(key);
                    if recent_keys.len() > 2 {
                        recent_keys.remove(0); // Keep only last two
                    }
                }
            }

            if recent_keys.len() == 2 {
                use egui::Key::*;
                let (a, b) = (recent_keys[0], recent_keys[1]);

                if ui.input(|i| i.key_down(egui::Key::Space)) {
                    match (a, b) {
                        (ArrowUp, ArrowRight) | (ArrowRight, ArrowUp) if directions.up_right => {
                            new_pos.0 -= 2;
                            new_pos.1 += 2; // Move up-right
                            recent_keys.clear();
                        }
                        (ArrowDown, ArrowRight) | (ArrowRight, ArrowDown)
                            if directions.down_right =>
                        {
                            new_pos.0 += 2;
                            new_pos.1 += 2; // Move down-right
                            recent_keys.clear();
                        }
                        (ArrowDown, ArrowLeft) | (ArrowLeft, ArrowDown) if directions.down_left => {
                            new_pos.0 += 2;
                            new_pos.1 -= 2; // Move down-left
                            recent_keys.clear();
                        }
                        (ArrowUp, ArrowLeft) | (ArrowLeft, ArrowUp) if directions.up_left => {
                            new_pos.0 -= 2;
                            new_pos.1 -= 2; // Move up-left
                            recent_keys.clear();
                        }
                        _ => {}
                    }
                } else {
                    match (a, b) {
                        (ArrowUp, ArrowRight) | (ArrowRight, ArrowUp) if directions.up_right => {
                            new_pos.0 -= 1;
                            new_pos.1 += 1; // Move up-right
                            recent_keys.clear();
                        }
                        (ArrowDown, ArrowRight) | (ArrowRight, ArrowDown)
                            if directions.down_right =>
                        {
                            new_pos.0 += 1;
                            new_pos.1 += 1; // Move down-right
                            recent_keys.clear();
                        }
                        (ArrowDown, ArrowLeft) | (ArrowLeft, ArrowDown) if directions.down_left => {
                            new_pos.0 += 1;
                            new_pos.1 -= 1; // Move down-left
                            recent_keys.clear();
                        }
                        (ArrowUp, ArrowLeft) | (ArrowLeft, ArrowUp) if directions.up_left => {
                            new_pos.0 -= 1;
                            new_pos.1 -= 1; // Move up-left
                            recent_keys.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {
            if ui.input(|i| i.key_down(egui::Key::Space)) {
                // Handle cardinal movement based on allowed directions
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    new_pos.0 -= 2; // Move up
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    new_pos.0 += 2; // Move down
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    new_pos.1 -= 2; // Move left
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    new_pos.1 += 2; // Move right
                }
            } else {
                // Handle cardinal movement based on allowed directions
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    new_pos.0 -= 1; // Move up
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    new_pos.0 += 1; // Move down
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    new_pos.1 -= 1; // Move left
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    new_pos.1 += 1; // Move right
                }
            }
        }
    }

    // Check if new position is valid
    if new_pos.0 >= 0
        && new_pos.0 < game.board_size.1 as isize
        && new_pos.1 >= 0
        && new_pos.1 < game.board_size.0 as isize
    {
        // if at end space, end the game
        if matches!(
            game.playing_board[new_pos.0 as usize][new_pos.1 as usize],
            Tile::EndSpace
        ) {
            game.mode = GameMode::Editing;
        }

        game.previous_player_pos = game.player_pos; // Store previous position for movement logic
        game.player_pos = (new_pos.0 as usize, new_pos.1 as usize);
    }
}

fn display_playing_board(ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.vertical(|ui| {
        if ui.button("Switch to Editing Mode").clicked() {
            game.mode = GameMode::Editing;
        }

        ui.add_space(50.0);

        // Display board
        egui::Grid::new("game_board")
            .spacing([1.0, 1.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                for row in 0..game.board_size.1 {
                    for col in 0..game.board_size.0 {
                        // Draw the tile and get its response
                        let response = draw_tile(
                            game,
                            game.editing_board.get_tile(row, col).unwrap(),
                            ui,
                            game.player_pos == (row, col),
                        );
                        if response.clicked() {
                            // Handle logic later
                        }
                        // Draw faint white border around each cell
                        let rect = response.rect;
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(0.5, egui::Color32::from_white_alpha(64)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    ui.end_row();
                }
            });
    });
}

fn draw_tile(game: &mut FoamGame, tile: &Tile, ui: &mut egui::Ui, player: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::Vec2 { x: 32.0, y: 32.0 }, egui::Sense::click());
    let painter = ui.painter_at(rect);

    // Draw the base tile image
    painter.image(
        game.get_texture(ui.ctx(), tile).unwrap().id(),
        rect,
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    // Draw overlays
    match tile {
        Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
            let center = rect.center();
            let offset = 10.0;
            let arrow_color = egui::Stroke::new(2.0, egui::Color32::BLACK);

            if directions.up {
                painter.arrow(center, egui::vec2(0.0, -offset), arrow_color);
            }
            if directions.right {
                painter.arrow(center, egui::vec2(offset, 0.0), arrow_color);
            }
            if directions.down {
                painter.arrow(center, egui::vec2(0.0, offset), arrow_color);
            }
            if directions.left {
                painter.arrow(center, egui::vec2(-offset, 0.0), arrow_color);
            }
        }
        Tile::MoveDiagonal(directions) => {
            let center = rect.center();
            let offset = 10.0;
            let arrow_color = egui::Stroke::new(2.0, egui::Color32::BLACK);

            if directions.up_right {
                painter.arrow(center, egui::vec2(offset, -offset), arrow_color);
            }
            if directions.down_right {
                painter.arrow(center, egui::vec2(offset, offset), arrow_color);
            }
            if directions.down_left {
                painter.arrow(center, egui::vec2(-offset, offset), arrow_color);
            }
            if directions.up_left {
                painter.arrow(center, egui::vec2(-offset, -offset), arrow_color);
            }
        }
        Tile::Bounce(val) => {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}", val),
                egui::FontId::monospace(16.0),
                egui::Color32::RED,
            );
        }
        Tile::Portal(c) => {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                c.to_string(),
                egui::FontId::monospace(30.0),
                egui::Color32::GREEN,
            );
        }
        _ => {}
    }

    // MARK: hehehehe HACKS
    if player {
        // Draw player position indicator as a red circle in top right corner
        let circle_radius = 8.0;
        let circle_center = egui::Pos2::new(rect.max.x - circle_radius, rect.min.y + circle_radius);
        painter.circle_filled(circle_center, circle_radius, egui::Color32::BLACK);
    }

    response.on_hover_text(tile.explanation())
}
