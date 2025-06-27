use std::{collections::HashMap, iter::FusedIterator};

use eframe::egui;
use egui::ImageButton;

struct FoamGame {
    startup: bool,
    editing_mode: bool,
    selected_type: Tile,
    board_size: (usize, usize),
    board: Vec<Vec<Tile>>,
    start_pos: Option<(usize, usize)>, // position of unique start tile
    end_pos: Option<(usize, usize)>,   // position of unique end tile
    player_pos: (usize, usize),

    texture_cache: HashMap<String, egui::TextureHandle>,
}

#[derive(Debug, Clone)]
struct DirectionsAllowed {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
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
                right: false,
                down: false,
                left: false,
            },
            remaining: 15,
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
            if self.inner.right {
                self.inner.right = false;
                if self.inner.down {
                    self.inner.down = false;
                    if self.inner.left {
                        self.inner.left = false;
                        unreachable!("Iterated through more than 15 directions");
                    } else {
                        self.inner.left = true;
                    }
                } else {
                    self.inner.down = true;
                }
            } else {
                self.inner.right = true;
            }
        } else {
            self.inner.up = true;
        }
        Some(self.inner.clone())
    }
}

impl FusedIterator for DirectionsAllowedIter {}

impl ExactSizeIterator for DirectionsAllowedIter {
    fn len(&self) -> usize {
        self.remaining
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

// Tiles:
// Each tile occupies one space on the board, and has different rules for movement
#[derive(Debug, Clone)]
enum Tile {
    Empty,
    MoveCardinal(DirectionsAllowed),
    // all other combinations of cardinal movement
    Bounce(isize), // Bounce some amount of squares, +/- some amount of acceleration or deceleration
    Portal,        // Portals from one tile to another
    Water,         // Water
    Ice,           // Ice
    Door,          // Doors, requires
    Wall,          // Blocks movement
    StartSpace,    // Start space, where the player starts
    EndSpace,      // End space, puzzle completion
    Cloud,         // Clouds, disappear after one use
}

fn all_tiles() -> impl Iterator<Item = Tile> {
    DirectionsAllowedIter::new()
        .map(Tile::MoveCardinal)
        .chain(vec![
            Tile::Bounce(1),
            Tile::Bounce(0),
            Tile::Bounce(-1),
            Tile::Portal,
            Tile::Water,
            Tile::Ice,
            Tile::Door,
            Tile::Wall,
            Tile::Cloud,
            Tile::StartSpace,
            Tile::EndSpace,
            Tile::Empty,
        ])
}

impl Tile {
    pub fn file_name(&self) -> String {
        match self {
            Tile::Empty => "assets/empty.png".to_string(),
            Tile::MoveCardinal(dirs) => {
                let mut file_name = "assets/cardinal".to_string();
                if dirs.up {
                    file_name.push_str("_up");
                }
                if dirs.right {
                    file_name.push_str("_right");
                }
                if dirs.down {
                    file_name.push_str("_down");
                }
                if dirs.left {
                    file_name.push_str("_left");
                }
                file_name.push_str(".png");
                file_name
            }
            Tile::Bounce(distance) => match distance {
                -1 => "assets/bounce_less.png".to_string(),
                0 => "assets/bounce_same.png".to_string(),
                1 => "assets/bounce_more.png".to_string(),
                _ => panic!("Bounce distance should be -1, 0, or 1: {}", distance),
            },
            Tile::Portal => "assets/portal.png".to_string(),
            Tile::Water => "assets/water.png".to_string(),
            Tile::Ice => "assets/ice.png".to_string(),
            Tile::Door => "assets/door.png".to_string(),
            Tile::Wall => "assets/wall.png".to_string(),
            Tile::StartSpace => "assets/start_space.png".to_string(),
            Tile::EndSpace => "assets/end_space.png".to_string(),
            Tile::Cloud => "assets/cloud.png".to_string(),
        }
    }

    pub fn explanation(&self) -> &str {
        match self {
            Tile::Empty => "An empty tile, no special properties.",
            Tile::MoveCardinal(_) => "A tile that allows movement up, down, left, and right.",
            Tile::Bounce(_) => "A tile that bounces the player a certain distance.",
            Tile::Portal => "A portal tile that teleports the player to another location.",
            Tile::Water => "A water tile, which may have special movement rules.",
            Tile::Ice => "An ice tile, which may affect movement speed.",
            Tile::Door => "A door tile, which may require a key or other condition to pass.",
            Tile::Wall => "A wall tile, which blocks movement.",
            Tile::StartSpace => "The starting space for the player.",
            Tile::EndSpace => "The end space for the puzzle completion.",
            Tile::Cloud => "A cloud tile that disappears after one use.",
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

        if !self.texture_cache.contains_key(&file_name) {
            // Load and cache the texture
            let image = image::ImageReader::open(&file_name)
                .map_err(|err| format!("Error loading texture file at {}: {}", file_name, err))?
                .decode()
                .map_err(|err| format!("Error decoding image at {}: {}", file_name, err))?;

            // Resize the image to 32x32
            let image = image.resize(32, 32, image::imageops::FilterType::Nearest);
            let size = [32, 32]; // Fixed size
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            let texture = ctx.load_texture(
                &file_name,
                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                egui::TextureOptions::default(),
            );

            self.texture_cache.insert(file_name.clone(), texture);
        }

        Ok(self.texture_cache.get(&file_name).unwrap())
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
}

fn editing_screen(ctx: &egui::Context, ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Editing Mode");

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI elements for editing the board
            if ui.button("Switch to Playing Mode").clicked() {
                game.editing_mode = !game.editing_mode;
            }

            // Display tile selection in a 3x15 grid
            let all_tiles = all_tiles().collect::<Vec<_>>();
            egui::Grid::new("tile_selector")
                .spacing([1.0, 1.0])
                .show(ui, |ui| {
                    for (index, tile_type) in all_tiles.iter().enumerate() {
                        if ui
                            .add(ImageButton::new(egui::Image::from_texture(
                                game.get_texture(ctx, tile_type).unwrap_or_else(|err| {
                                    panic!("Error loading texture: {}", err);
                                }),
                            )))
                            .clicked()
                        {
                            game.selected_type = tile_type.clone();
                        }

                        if (index + 1) % 15 == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.label("Selected Tile:")
                .on_hover_text(game.selected_type.explanation());
            let selected_tile = game.selected_type.clone();
            ui.add(egui::Image::new(
                game.get_texture(ctx, &selected_tile).unwrap(),
            ));
        })
    });

    // Create a container for modifications
    let mut modifications = Vec::new();

    // Display the board and allow tile selection
    egui::Grid::new("editing_board")
        .spacing([1.0, 1.0])
        .show(ui, |ui| {
            for (row_idx, row) in game.board.clone().iter().enumerate() {
                for (col_idx, tile) in row.iter().enumerate() {
                    // Create a button with color based on the tile type

                    if ui
                        .add(ImageButton::new(egui::Image::from_texture(
                            game.get_texture(ctx, tile).unwrap(),
                        )))
                        .clicked()
                    // Click and drag doesn't work (button steals focus I think)
                    {
                        // Collect modification for later application
                        modifications.push((row_idx, col_idx));
                    }
                }
                ui.end_row();
            }
        });

    // Apply all modifications after iteration is complete
    for (row_idx, col_idx) in modifications {
        game.board[row_idx][col_idx] = game.selected_type.clone();
    }
}

fn play_screen(ctx: &egui::Context, ui: &mut egui::Ui, game: &mut FoamGame) {
    ui.label("Playing Mode");

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI elements for playing the game
            if ui.button("Switch to Editing Mode").clicked() {
                game.editing_mode = !game.editing_mode;
            }

            ui.label("Player Position:")
                .on_hover_text("The current position of the player");
            ui.label(format!("({}, {})", game.player_pos.0, game.player_pos.1));
        });

        // Display the board and player position

        // Use Grid layout for proper row/column structure
        egui::Grid::new("game_board")
            .spacing([1.0, 1.0])
            .show(ui, |ui| {
                for row in game.board.clone().iter() {
                    for tile in row.iter() {
                        // Create a button with color based on the tile type
                        if ui
                            .add(ImageButton::new(egui::Image::from_texture(
                                game.get_texture(ctx, tile).unwrap(),
                            )))
                            .clicked()
                        {
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
