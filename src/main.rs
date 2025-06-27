use eframe::egui;
use egui::ImageButton;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

struct FoamGame {
    startup: bool,
    editing_mode: bool,
    selected_type: TileType,
    board_size: (usize, usize),
    board: Vec<Vec<TileType>>,
    has_start: bool,
    has_end: bool,
    player_pos: (usize, usize),
}

// Tiles:
// Each tile occupies one space on the board, and has different rules for movement
#[derive(Debug, Clone, EnumIter)]
enum TileType {
    Empty,
    MoveRight,
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

impl TileType {
    pub fn texture(&self, ctx: &egui::Context) -> Result<egui::TextureHandle, image::ImageError> {
        let file_name = match self {
            TileType::Empty => "assets/empty.png",
            TileType::MoveRight => "assets/right.png",
            // Add other tile types and their corresponding image files here
            _ => "assets/empty.png", // Fallback image
        };

        let image = image::ImageReader::open(file_name)?.decode()?;
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();

        Ok(ctx.load_texture(
            file_name,
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
            egui::TextureOptions::default(),
        ))
    }

    pub fn explanation(&self) -> &str {
        match self {
            TileType::Empty => "An empty tile, no special properties.",
            TileType::MoveRight => "A tile that allows movement to the right.",
            TileType::Bounce(_) => "A tile that bounces the player a certain distance.",
            TileType::Portal => "A portal tile that teleports the player to another location.",
            TileType::Water => "A water tile, which may have special movement rules.",
            TileType::Ice => "An ice tile, which may affect movement speed.",
            TileType::Door => "A door tile, which may require a key or other condition to pass.",
            TileType::Wall => "A wall tile, which blocks movement.",
            TileType::StartSpace => "The starting space for the player.",
            TileType::EndSpace => "The end space for the puzzle completion.",
            TileType::Cloud => "A cloud tile that disappears after one use.",
        }
    }
}

impl Default for FoamGame {
    fn default() -> Self {
        FoamGame {
            startup: true,
            editing_mode: false,
            selected_type: TileType::Empty,
            board_size: (0, 0),
            board: vec![],
            has_start: false,
            has_end: false,
            player_pos: (0, 0),
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
        game.board = vec![vec![TileType::Empty; game.board_size.0]; game.board_size.1];
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

            for tile_type in TileType::iter() {
                if ui
                    .add(ImageButton::new(egui::Image::from_texture(
                        &tile_type.texture(ctx).unwrap(),
                    )))
                    .clicked()
                {
                    game.selected_type = tile_type.clone();
                }
            }

            ui.label("Selected Tile:")
                .on_hover_text(game.selected_type.explanation());
            ui.add(egui::Image::new(&game.selected_type.texture(ctx).unwrap()));
        })
    });

    // Create a container for modifications
    let mut modifications = Vec::new();

    // Display the board and allow tile selection
    egui::Grid::new("editing_board")
        .spacing([0.0, 10.0])
        .show(ui, |ui| {
            for (row_idx, row) in game.board.iter().enumerate() {
                for (col_idx, tile) in row.iter().enumerate() {
                    // Create a button with color based on the tile type

                    if ui
                        .add(ImageButton::new(egui::Image::from_texture(
                            &tile.texture(ctx).unwrap(),
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
            .spacing([2.0, 2.0])
            .show(ui, |ui| {
                for row in game.board.iter() {
                    for tile in row.iter() {
                        // Create a button with color based on the tile type
                        if ui
                            .add(ImageButton::new(egui::Image::from_texture(
                                &tile.texture(ctx).unwrap(),
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
    options.viewport.resizable = Some(true); // idk if resizable good or not

    let _ = eframe::run_native(
        "Foam Game",
        options,
        Box::new(|_cc| Ok(Box::new(FoamGame::default()))),
    );
}
