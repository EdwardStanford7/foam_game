use eframe::egui;
use native_dialog::FileDialog;
use std::collections::HashMap;

struct FoamGame {
    startup: bool,
    editing_mode: bool,
    selected_type: Tile,
    board_size: (usize, usize),
    board: Vec<Vec<Tile>>,
    start_pos: Option<(usize, usize)>, // position of unique start tile
    end_pos: Option<(usize, usize)>,   // position of unique end tile
    player_pos: (usize, usize),
    texture_cache: HashMap<String, egui::TextureHandle>, // Cache for textures to avoid reloading them every frame
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
struct DirectionsAllowed {
    up: bool,
    up_right: bool,
    right: bool,
    down_right: bool,
    down: bool,
    down_left: bool,
    left: bool,
    up_left: bool,
}

struct DirectionsAllowedIter {
    inner: DirectionsAllowed,
    remaining: usize,
}

impl DirectionsAllowedIter {
    pub fn new() -> Self {
        DirectionsAllowedIter {
            inner: DirectionsAllowed {
                up: false,
                up_right: false,
                right: false,
                down_right: false,
                down: false,
                down_left: false,
                left: false,
                up_left: false,
            },
            remaining: 255, // 2^8 - 1 = 255, all combinations of 8 directions
        }
    }
}

impl Iterator for DirectionsAllowedIter {
    type Item = DirectionsAllowed;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;

        if self.inner.up {
            self.inner.up = false;
            if self.inner.up_right {
                self.inner.up_right = false;
                if self.inner.right {
                    self.inner.right = false;
                    if self.inner.down_right {
                        self.inner.down_right = false;
                        if self.inner.down {
                            self.inner.down = false;
                            if self.inner.down_left {
                                self.inner.down_left = false;
                                if self.inner.left {
                                    self.inner.left = false;
                                    if self.inner.up_left {
                                        self.inner.up_left = false;
                                        unreachable!("Iterated through more than 15 directions");
                                    } else {
                                        self.inner.up_left = true;
                                    }
                                } else {
                                    self.inner.left = true;
                                }
                            } else {
                                self.inner.down_left = true;
                            }
                        } else {
                            self.inner.down = true;
                        }
                    } else {
                        self.inner.down_right = true;
                    }
                } else {
                    self.inner.right = true;
                }
            } else {
                self.inner.up_right = true;
            }
        } else {
            self.inner.up = true;
        }
        Some(self.inner.clone())
    }
}

// struct Direction(isize, isize);

// use std::ops::Index;

// impl Index<Direction> for DirectionsAllowed {
//     type Output = bool;

//     fn index(&self, index: Direction) -> &Self::Output {
//         match index {
//             Direction(0, 1) => &self.up,
//             Direction(0, -1) => &self.down,
//             Direction(-1, 0) => &self.left,
//             Direction(1, 0) => &self.right,
//             _ => panic!("Invalid direction"),
//         }
//     }
// }

// Each tile occupies one space on the board, and has different rules for movement
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
enum Tile {
    Empty,
    Move(DirectionsAllowed),
    Cloud(DirectionsAllowed), // Clouds, disappear after one use
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
    DirectionsAllowedIter::new()
        .map(Tile::Move)
        .chain(DirectionsAllowedIter::new().map(Tile::Cloud))
        .chain(vec![
            Tile::Bounce(1),
            Tile::Bounce(0),
            Tile::Bounce(-1),
            Tile::Portal('A'),
            Tile::Water,
            Tile::Ice,
            Tile::Door,
            Tile::Wall,
            Tile::StartSpace,
            Tile::EndSpace,
            Tile::Empty,
        ])
}

impl Tile {
    pub fn file_name(&self) -> &str {
        match self {
            Tile::Empty => "assets/empty.png",
            Tile::Move(_) => "assets/move.png",
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
            Tile::Move(_) => "A tile that allows movement in specific directions.",
            Tile::Cloud(_) => "A cloud tile that disappears after one use.",
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

    fn draw_tile(&mut self, ui: &mut egui::Ui, tile: &Tile) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2 { x: 32.0, y: 32.0 }, egui::Sense::click());
        let painter = ui.painter_at(rect);

        // Draw the base tile image
        painter.image(
            self.get_texture(ui.ctx(), tile).unwrap().id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        // Draw overlays
        match tile {
            Tile::Move(directions) | Tile::Cloud(directions) => {
                let center = rect.center();
                let offset = 10.0;

                let arrow_color = egui::Stroke::new(2.0, egui::Color32::BLACK);

                if directions.up {
                    painter.arrow(center, egui::vec2(0.0, -offset), arrow_color);
                }
                if directions.up_right {
                    painter.arrow(center, egui::vec2(offset, -offset), arrow_color);
                }
                if directions.right {
                    painter.arrow(center, egui::vec2(offset, 0.0), arrow_color);
                }
                if directions.down_right {
                    painter.arrow(center, egui::vec2(offset, offset), arrow_color);
                }
                if directions.down {
                    painter.arrow(center, egui::vec2(0.0, offset), arrow_color);
                }
                if directions.down_left {
                    painter.arrow(center, egui::vec2(-offset, offset), arrow_color);
                }
                if directions.left {
                    painter.arrow(center, egui::vec2(-offset, 0.0), arrow_color);
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
                    egui::FontId::proportional(16.0),
                    egui::Color32::YELLOW,
                );
            }
            _ => {}
        }

        response
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
        let board_data = serde_json::to_string(&self.board)
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
        self.board = serde_json::from_str(&board_data)
            .map_err(|err| format!("Error deserializing board data: {}", err))?;

        Ok(())
    }

    fn is_valid_board(&self) -> bool {
        // Check if the board has a start and end tile
        let has_start = self.board.iter().any(|row| row.contains(&Tile::StartSpace));
        let has_end = self.board.iter().any(|row| row.contains(&Tile::EndSpace));
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
            board_size: (0, 0),
            board: vec![],
            start_pos: None,
            end_pos: None,
            player_pos: (0, 0),
            texture_cache: HashMap::new(),
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
        ui.label("Width:");
        ui.add(egui::Slider::new(&mut game.board_size.0, 5..=40).integer());
    });

    ui.horizontal(|ui| {
        ui.label("Height:");
        ui.add(egui::Slider::new(&mut game.board_size.1, 5..=20).integer());
    });

    if ui.button("Start Editing").clicked() {
        // Initialize the board with the selected size
        game.board = vec![vec![Tile::Empty; game.board_size.0]; game.board_size.1];
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

    // Display menus and buttons for editing the board
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI buttons to change modes and save/load the board
            if ui.button("Switch to Playing Mode").clicked() && game.is_valid_board() {
                game.editing_mode = false;
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
            game.draw_tile(ui, &selected_tile);
        });

        let all_tiles = all_tiles().collect::<Vec<_>>();
        egui::Grid::new("tile_selector")
            .spacing([1.0, 1.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                for (index, tile) in all_tiles.iter().enumerate() {
                    if game.draw_tile(ui, tile).clicked() {
                        game.selected_type = tile.clone();
                    }

                    if (index + 1) % 30 == 0 {
                        ui.end_row();
                    }
                }
            });
    });

    // empty space to separate the menus from the board
    ui.add_space(25.0);

    // Create a container for modifications
    let mut modifications = Vec::new();

    // Display the board directly in the parent container, without an outer border
    egui::Grid::new("editing_board")
        .spacing([1.0, 1.0])
        .min_col_width(0.0)
        .show(ui, |ui| {
            for (row_idx, row) in game.board.clone().iter().enumerate() {
                for (col_idx, tile) in row.iter().enumerate() {
                    let response = game.draw_tile(ui, tile);
                    if response.clicked() {
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
                            *current_pos = Some((row_idx, col_idx));
                        }

                        modifications.push((row_idx, col_idx, game.selected_type.clone()));
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
        game.board[row_idx][col_idx] = tile;
    }
}

fn play_screen(_ctx: &egui::Context, ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Playing Mode");

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI elements for playing the game
            if ui.button("Switch to Editing Mode").clicked() {
                game.editing_mode = true;
            }

            ui.label("Player Position:")
                .on_hover_text("The current position of the player");
            ui.label(format!("({}, {})", game.player_pos.0, game.player_pos.1));
        });

        // Display the board and player position

        // Use Grid layout for proper row/column structure
        egui::Grid::new("game_board")
            .spacing([1.0, 1.0])
            .min_col_width(0.0)
            .show(ui, |ui| {
                for row in game.board.clone().iter() {
                    for tile in row.iter() {
                        // Create a button with color based on the tile type
                        if game.draw_tile(ui, tile).clicked() {
                            // Handle logic later
                        }
                    }
                    ui.end_row();
                }
            });
    });
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
