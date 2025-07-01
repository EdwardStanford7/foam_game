use eframe::egui;
use native_dialog::FileDialog;
use std::collections::HashMap;

struct FoamGame {
    startup: bool,
    editing_mode: bool,
    selected_type: Tile,
    selected_tile_pos: Option<(usize, usize)>, // Currently selected tile position for editing
    board_size: (usize, usize),
    editing_board: Vec<Vec<Tile>>,
    playing_board: Vec<Vec<Tile>>,
    start_pos: Option<(usize, usize)>, // position of unique start tile
    end_pos: Option<(usize, usize)>,   // position of unique end tile
    player_pos: (usize, usize),
    previous_player_pos: (usize, usize), // Store previous player position for movement logic
    texture_cache: HashMap<String, egui::TextureHandle>, // Cache for textures to avoid reloading them every frame
    recent_keys: Vec<egui::Key>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
struct CardinalDirectionsAllowed {
    up: bool,
    right: bool,
    down: bool,
    left: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
struct DiagonalDirectionsAllowed {
    up_right: bool,
    down_right: bool,
    down_left: bool,
    up_left: bool,
}

// Each tile occupies one space on the board, and has different rules for movement
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
enum Tile {
    Empty,
    MoveCardinal(CardinalDirectionsAllowed),
    MoveDiagonal(DiagonalDirectionsAllowed), // Move in specific directions, can be cardinal or diagonal
    Cloud(CardinalDirectionsAllowed),        // Clouds, disappear after one use
    Bounce(isize), // Bounce some amount of squares, +/- some amount of acceleration or deceleration
    Portal(char),  // Portal, teleport to other portal with same letter
    Water,         // Water
    Ice,           // Ice
    Door,          // Doors, requires
    Wall,          // Blocks movement
    StartSpace,    // Start space, where the player starts
    EndSpace,      // End space, puzzle completion
}

fn all_tiles() -> impl Iterator<Item = Tile> {
    vec![
        Tile::MoveCardinal(CardinalDirectionsAllowed {
            up: true,
            right: true,
            down: true,
            left: true,
        }),
        Tile::MoveDiagonal(DiagonalDirectionsAllowed {
            up_right: true,
            down_right: true,
            down_left: true,
            up_left: true,
        }),
        Tile::Cloud(CardinalDirectionsAllowed {
            up: true,
            right: true,
            down: true,
            left: true,
        }),
        Tile::Bounce(0),
        Tile::Portal('A'),
        Tile::Water,
        Tile::Ice,
        Tile::Door,
        Tile::Wall,
        Tile::StartSpace,
        Tile::EndSpace,
        Tile::Empty,
    ]
    .into_iter()
}

impl Tile {
    pub fn file_name(&self) -> &str {
        match self {
            Tile::Empty => "assets/empty.png",
            Tile::MoveCardinal(_) => "assets/move_cardinal.png",
            Tile::MoveDiagonal(_) => "assets/move_diagonal.png",
            Tile::Cloud(_) => "assets/cloud.png",
            Tile::Bounce(_) => "assets/bounce.png",
            Tile::Portal(_) => "assets/portal.png",
            Tile::Water => "assets/water.png",
            Tile::Ice => "assets/ice.png",
            Tile::Door => "assets/door.png",
            Tile::Wall => "assets/wall.png",
            Tile::StartSpace => "assets/start_space.png",
            Tile::EndSpace => "assets/end_space.png",
        }
    }

    pub fn explanation(&self) -> &str {
        match self {
            Tile::Empty => "An empty tile, no special properties.",
            Tile::MoveCardinal(_) => {
                "A tile that allows moving up, down, left, right. Use arrow keys to toggle directions."
            }
            Tile::MoveDiagonal(_) => {
                "A tile that allows moving up-right, down-right, down-left, up-left. Use arrow keys to toggle directions."
            }
            Tile::Cloud(_) => {
                "A cloud tile that disappears after one use. Use arrow keys to toggle directions."
            }
            Tile::Bounce(_) => "A tile that bounces the player a certain distance.",
            Tile::Portal(_) => "A portal tile that teleports the player to another location.",
            Tile::Water => "A water tile, which may have special movement rules.",
            Tile::Ice => "An ice tile, which may affect movement speed.",
            Tile::Door => "A door tile, which may require a key or other condition to pass.",
            Tile::Wall => "A wall tile, which blocks movement.",
            Tile::StartSpace => "The starting space for the player.",
            Tile::EndSpace => "The end space for the puzzle completion.",
        }
    }
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
            serde_json::to_string(&(&self.editing_board, self.start_pos, self.end_pos))
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
        let (board, start_pos, end_pos): (
            Vec<Vec<Tile>>,
            Option<(usize, usize)>,
            Option<(usize, usize)>,
        ) = serde_json::from_str(&board_data)
            .map_err(|err| format!("Error deserializing board data: {}", err))?;

        self.editing_board = board;
        self.start_pos = start_pos;
        self.end_pos = end_pos;

        Ok(())
    }

    fn is_valid_board(&self) -> bool {
        // Check if the board has a start and end tile
        let has_start = self
            .editing_board
            .iter()
            .any(|row| row.contains(&Tile::StartSpace));
        let has_end = self
            .editing_board
            .iter()
            .any(|row| row.contains(&Tile::EndSpace));
        has_start && has_end

        // todo later check things like matching portal pairs, etc.
    }
}

impl Default for FoamGame {
    fn default() -> Self {
        FoamGame {
            startup: true,
            editing_mode: false,
            selected_type: Tile::Empty,
            selected_tile_pos: None,
            board_size: (0, 0),
            editing_board: vec![],
            playing_board: vec![],
            start_pos: None,
            end_pos: None,
            player_pos: (0, 0),
            previous_player_pos: (0, 0),
            texture_cache: HashMap::new(),
            recent_keys: Vec::new(),
        }
    }
}

impl eframe::App for FoamGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.startup {
                startup_screen(ui, self);
            } else if self.editing_mode {
                editing_screen(ctx, ui, self);
            } else {
                play_screen(ctx, ui, self);
            }
        });
    }
}

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
        game.editing_board = vec![vec![Tile::Empty; game.board_size.0]; game.board_size.1];
        // Switch to editing mode
        game.editing_mode = true;
        // Exit startup screen
        game.startup = false;
    }

    if ui.button("Load Board").clicked() {
        // Load board from file
        if let Err(err) = game.load_board() {
            ui.label(format!("Error loading board: {}", err));
        } else {
            game.editing_mode = true;
            game.startup = false;
        }
    }
}

fn editing_screen(_ctx: &egui::Context, ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Editing Mode");
    display_editing_menu(ui, game);
    ui.add_space(25.0);
    display_editing_board(ui, game);

    // Keyboard input for tile modification.
    if game.selected_tile_pos.is_none() {
        return; // No tile selected, skip input handling
    }
    editing_keyboard_input(ui, game);
}

fn editing_keyboard_input(ui: &mut egui::Ui, game: &mut FoamGame) {
    match &mut game.editing_board[game.selected_tile_pos.unwrap().0]
        [game.selected_tile_pos.unwrap().1]
    {
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
                    let response =
                        draw_tile(game, &game.editing_board[row][col].clone(), ui, false);
                    if response.clicked() {
                        // Update the selected tile
                        game.selected_tile_pos = Some((row, col));

                        if std::mem::discriminant(&game.selected_type)
                            == std::mem::discriminant(&game.editing_board[row][col])
                        {
                            continue; // Skip modification for this tile
                        }

                        // Collect modification for later application
                        // Handle unique tiles (StartSpace and EndSpace)
                        if matches!(game.selected_type, Tile::StartSpace | Tile::EndSpace) {
                            let (current_pos, _) = match game.selected_type {
                                Tile::StartSpace => (&mut game.start_pos, true),
                                Tile::EndSpace => (&mut game.end_pos, false),
                                _ => unreachable!(),
                            };

                            if let Some(pos) = current_pos.take() {
                                modifications.push((pos.0, pos.1, Tile::Empty));
                            }
                            *current_pos = Some((row, col));
                        }

                        modifications.push((row, col, game.selected_type.clone()));
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
        game.editing_board[row_idx][col_idx] = tile;
    }
}

fn display_editing_menu(ui: &mut egui::Ui, game: &mut FoamGame) {
    // Display menus and buttons for editing the board
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI buttons to change modes and save/load the board
            if ui.button("Switch to Playing Mode").clicked() && game.is_valid_board() {
                game.editing_mode = false;
                game.player_pos = game.start_pos.unwrap();
                game.playing_board = game.editing_board.clone();
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
            ui.label("Selected Tile:")
                .on_hover_text(game.selected_type.explanation());

            let selected_tile = game.selected_type.clone();

            draw_tile(game, &selected_tile, ui, false);
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            for tile in all_tiles() {
                if draw_tile(game, &tile, ui, false).clicked() {
                    game.selected_type = tile;
                }
            }
        });
    });
}

fn play_screen(_ctx: &egui::Context, ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Playing Mode");

    display_playing_board(ui, game);

    handle_player_movement(ui, game);
}

fn handle_player_movement(ui: &mut egui::Ui, game: &mut FoamGame) {
    // If moving from a cloud tile, remove it
    if matches!(
        game.playing_board[game.previous_player_pos.0][game.previous_player_pos.1],
        Tile::Cloud(_)
    ) && game.previous_player_pos != game.player_pos
    {
        game.playing_board[game.previous_player_pos.0][game.previous_player_pos.1] = Tile::Empty;
    }

    let current_tile = &game.playing_board[game.player_pos.0][game.player_pos.1];
    let mut new_pos: (isize, isize) = (game.player_pos.0 as isize, game.player_pos.1 as isize);

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
                    game.recent_keys.push(key);
                    if game.recent_keys.len() > 2 {
                        game.recent_keys.remove(0); // Keep only last two
                    }
                }
            }

            if game.recent_keys.len() == 2 {
                use egui::Key::*;
                let (a, b) = (game.recent_keys[0], game.recent_keys[1]);

                if ui.input(|i| i.key_down(egui::Key::Space)) {
                    match (a, b) {
                        (ArrowUp, ArrowRight) | (ArrowRight, ArrowUp) if directions.up_right => {
                            new_pos.0 -= 2;
                            new_pos.1 += 2; // Move up-right
                            game.recent_keys.clear();
                        }
                        (ArrowDown, ArrowRight) | (ArrowRight, ArrowDown)
                            if directions.down_right =>
                        {
                            new_pos.0 += 2;
                            new_pos.1 += 2; // Move down-right
                            game.recent_keys.clear();
                        }
                        (ArrowDown, ArrowLeft) | (ArrowLeft, ArrowDown) if directions.down_left => {
                            new_pos.0 += 2;
                            new_pos.1 -= 2; // Move down-left
                            game.recent_keys.clear();
                        }
                        (ArrowUp, ArrowLeft) | (ArrowLeft, ArrowUp) if directions.up_left => {
                            new_pos.0 -= 2;
                            new_pos.1 -= 2; // Move up-left
                            game.recent_keys.clear();
                        }
                        _ => {}
                    }
                } else {
                    match (a, b) {
                        (ArrowUp, ArrowRight) | (ArrowRight, ArrowUp) if directions.up_right => {
                            new_pos.0 -= 1;
                            new_pos.1 += 1; // Move up-right
                            game.recent_keys.clear();
                        }
                        (ArrowDown, ArrowRight) | (ArrowRight, ArrowDown)
                            if directions.down_right =>
                        {
                            new_pos.0 += 1;
                            new_pos.1 += 1; // Move down-right
                            game.recent_keys.clear();
                        }
                        (ArrowDown, ArrowLeft) | (ArrowLeft, ArrowDown) if directions.down_left => {
                            new_pos.0 += 1;
                            new_pos.1 -= 1; // Move down-left
                            game.recent_keys.clear();
                        }
                        (ArrowUp, ArrowLeft) | (ArrowLeft, ArrowUp) if directions.up_left => {
                            new_pos.0 -= 1;
                            new_pos.1 -= 1; // Move up-left
                            game.recent_keys.clear();
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
            game.editing_mode = true; // Switch back to editing mode
        }

        game.previous_player_pos = game.player_pos; // Store previous position for movement logic
        game.player_pos = (new_pos.0 as usize, new_pos.1 as usize);
    }
}

fn display_playing_board(ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.vertical(|ui| {
        if ui.button("Switch to Editing Mode").clicked() {
            game.editing_mode = true;
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
                            &game.playing_board[row][col].clone(),
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

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.viewport.resizable = Some(true);

    options.viewport.inner_size = Some(egui::vec2(1600.0, 900.0));

    let _ = eframe::run_native(
        "Foam Game",
        options,
        Box::new(|_cc| Ok(Box::new(FoamGame::default()))),
    );
}
