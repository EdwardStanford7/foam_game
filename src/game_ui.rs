//!
//! Logic for displaying the game UI and handling user input
//!

use super::editing_model::EditingModel;
use super::playing_model::PlayingModel;
use super::tile::{ALL_TILES, Tile};
use eframe::egui;
use native_dialog::FileDialog;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum AppMode {
    Startup,
    Editing,
    Playing,
}

pub struct App {
    editing_model: EditingModel, // Struct that contains actual game data and logic
    playing_model: PlayingModel, // Struct that contains game data and logic for playing mode

    mode: AppMode,
    selected_type: Tile,
    selected_tile_pos: Option<(usize, usize)>, // Currently selected tile position for editing
    width_slider: usize,                       // Width slider for board size
    height_slider: usize,                      // Height slider for board size

    /// Keys pending in the last key window buffer
    pending_keys: Vec<egui::Key>,
    /// Completed window of keys pressed
    recent_keys: Vec<egui::Key>, // Keys that have been processed and are
    /// Time that last keypress window was opened
    last_keyboard_window: f64, // Last time the keyboard window was updated
}

lazy_static::lazy_static! {
    static ref TEXTURE_CACHE: Mutex<HashMap<String, egui::TextureHandle>> = Mutex::new(HashMap::new());
}

// Add method to get cached texture
fn get_texture(ctx: &egui::Context, tile: &Tile) -> Result<egui::TextureHandle, String> {
    let file_name = tile.file_name();

    let mut cache = TEXTURE_CACHE.lock().unwrap();

    if !cache.contains_key(file_name) {
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

        cache.insert(file_name.to_string(), texture);
    }

    Ok(cache.get(file_name).unwrap().clone())
}

impl Default for App {
    fn default() -> Self {
        App {
            editing_model: Default::default(),
            playing_model: Default::default(),
            mode: AppMode::Startup,
            selected_type: Tile::Empty,
            selected_tile_pos: None,
            width_slider: 0,
            height_slider: 0,
            pending_keys: Vec::new(),
            recent_keys: Vec::new(),
            last_keyboard_window: 0.0, // Initialize last keyboard window time
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            update_recent_keys(ui, self);
            match self.mode {
                AppMode::Startup => startup_screen(ui, self),
                AppMode::Editing => editing_screen(ui, self),
                AppMode::Playing => play_screen(ui, self),
            }
        });
    }
}

/*
    Key enum & key logic
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionKey {
    Up,
    Right,
    Down,
    Left,
    UpRight,
    DownRight,
    DownLeft,
    UpLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectionKeyWithJump {
    pub direction: DirectionKey,
    pub move_speed: usize, // Number of tiles to move in the given direction
}

pub fn direction_key_from_bools(
    up: bool,
    right: bool,
    down: bool,
    left: bool,
    jump: usize,
) -> Option<DirectionKeyWithJump> {
    let direction = match (up, right, down, left) {
        (true, false, false, false) => Some(DirectionKey::Up),
        (false, true, false, false) => Some(DirectionKey::Right),
        (false, false, true, false) => Some(DirectionKey::Down),
        (false, false, false, true) => Some(DirectionKey::Left),
        (true, true, false, false) => Some(DirectionKey::UpRight),
        (false, true, true, false) => Some(DirectionKey::DownRight),
        (false, false, true, true) => Some(DirectionKey::DownLeft),
        (true, false, false, true) => Some(DirectionKey::UpLeft),
        _ => None,
    }?;

    Some(DirectionKeyWithJump {
        direction,
        move_speed: jump,
    })
}

pub fn direction_key_into_bools(
    keypress: &DirectionKeyWithJump,
) -> (bool, bool, bool, bool, usize) {
    let mut up = false;
    let mut right = false;
    let mut down = false;
    let mut left = false;

    match keypress.direction {
        DirectionKey::Up => up = true,
        DirectionKey::Right => right = true,
        DirectionKey::Down => down = true,
        DirectionKey::Left => left = true,
        DirectionKey::UpRight => {
            up = true;
            right = true;
        }
        DirectionKey::DownRight => {
            down = true;
            right = true;
        }
        DirectionKey::DownLeft => {
            down = true;
            left = true;
        }
        DirectionKey::UpLeft => {
            up = true;
            left = true;
        }
    }

    let move_speed = keypress.move_speed;

    (up, right, down, left, move_speed)
}

pub fn direction_key_from_egui_keys(keys: &[egui::Key]) -> Option<DirectionKeyWithJump> {
    let mut move_speed = 1;
    let mut up = false;
    let mut right = false;
    let mut down = false;
    let mut left = false;

    for &key in keys {
        match key {
            egui::Key::ArrowUp => up = true,
            egui::Key::ArrowRight => right = true,
            egui::Key::ArrowDown => down = true,
            egui::Key::ArrowLeft => left = true,
            egui::Key::Space => move_speed = 2, // Space key indicates a move speed of 2
            _ => {
                // Ignore other keys
                continue;
            }
        }
    }

    direction_key_from_bools(up, right, down, left, move_speed)
}

impl App {
    /// Get keys pressed (with exactly-once semantics, clearing them)
    /// Returns Some(nonempty vec) or None
    pub fn get_keys_pressed(&mut self) -> Option<DirectionKeyWithJump> {
        let result = std::mem::take(&mut self.recent_keys);
        direction_key_from_egui_keys(&result)
    }
}

const KEY_WINDOW_BUFFER_SECS: f64 = 0.1;

/// Get keyboard input from egui and load it into recent_keys
fn update_recent_keys(ui: &mut egui::Ui, app: &mut App) {
    // Add any new key presses this frame
    ui.input(|i| {
        for key in [
            egui::Key::ArrowUp,
            egui::Key::ArrowRight,
            egui::Key::ArrowDown,
            egui::Key::ArrowLeft,
            egui::Key::Space, // Space for jump
        ] {
            if i.key_pressed(key) {
                if app.pending_keys.is_empty() {
                    app.last_keyboard_window = i.time;
                }
                app.pending_keys.push(key);
            }
        }
    });

    if !app.pending_keys.is_empty()
        && app.recent_keys.is_empty()
        && ui.input(|i| i.time) - app.last_keyboard_window > KEY_WINDOW_BUFFER_SECS
    {
        std::mem::swap(&mut app.recent_keys, &mut app.pending_keys);
    }
}

/*
    Draw tile
*/

fn draw_tile(tile: &Tile, ui: &mut egui::Ui, player: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::Vec2 { x: 32.0, y: 32.0 }, egui::Sense::click());
    let painter = ui.painter_at(rect);

    // Draw the base tile image
    painter.image(
        get_texture(ui.ctx(), tile).unwrap().id(),
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

    if player {
        // Draw player position indicator as a red circle in top right corner
        let circle_radius = 8.0;
        let circle_center = egui::Pos2::new(rect.max.x - circle_radius, rect.min.y + circle_radius);
        painter.circle_filled(circle_center, circle_radius, egui::Color32::BLACK);
    }

    response.on_hover_text(tile.explanation())
}

/*
    Startup mode
*/

fn startup_screen(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Welcome to Foam Game!");

    // Board size selection
    ui.label("Select board size:");

    ui.horizontal(|ui| {
        ui.label("Width:");
        ui.add(egui::Slider::new(&mut app.width_slider, 5..=40).integer());
    });

    ui.horizontal(|ui| {
        ui.label("Height:");
        ui.add(egui::Slider::new(&mut app.height_slider, 5..=20).integer());
    });

    if ui.button("Start Editing").clicked() {
        // Initialize the board with the selected size
        app.editing_model = EditingModel::new((app.width_slider, app.height_slider));
        app.mode = AppMode::Editing;
    }

    if ui.button("Load Board").clicked() {
        // Load board from file
        let filename = open_file_dialog(false);
        if filename.is_err() {
            return;
        }

        let model = EditingModel::load_board(filename.unwrap().as_str());

        if model.is_ok() {
            app.editing_model = model.unwrap();
            app.mode = AppMode::Editing;
        }
    }
}

fn open_file_dialog(is_save: bool) -> Result<String, String> {
    let dialog = FileDialog::new().add_filter("Foam Game Board", &["fgb"]);

    let file_path = if is_save {
        dialog.set_title("Save Board").show_save_single_file()
    } else {
        dialog.set_title("Load Board").show_open_single_file()
    };

    Ok(file_path
        .ok()
        .flatten()
        .ok_or("No file selected".to_string())?
        .to_string_lossy()
        .to_string())
}

/*
    Editing mode
*/

fn editing_screen(ui: &mut egui::Ui, app: &mut App) {
    ui.label("Editing Mode");
    display_editing_menu(ui, app);
    ui.add_space(25.0);
    display_editing_board(ui, app);

    if let Some(keypress) = app.get_keys_pressed() {
        if let Some(selected_tile_pos) = app.selected_tile_pos {
            app.editing_model.edit_tile(selected_tile_pos, &keypress);
        }
    }
}

fn display_editing_menu(ui: &mut egui::Ui, app: &mut App) {
    // Display menus and buttons for editing the board
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            // Add UI buttons to change modes and save/load the board
            if ui.button("Switch to Playing Mode").clicked()
                && app.editing_model.board_is_playable()
            {
                app.mode = AppMode::Playing;
                app.playing_model = PlayingModel::new(&app.editing_model); // Initialize playing model
            }
            if ui.button("Save Board").clicked() {
                let file_name = open_file_dialog(true);
                if let Ok(file_name) = file_name {
                    let _ = app.editing_model.save_board(file_name.as_str());
                }
            }
            if ui.button("Load Board").clicked() {
                let file_name = open_file_dialog(false);
                if let Ok(file_name) = file_name {
                    let model = EditingModel::load_board(file_name.as_str());
                    if model.is_ok() {
                        app.editing_model = model.unwrap();
                    }
                }
            }
            ui.label("Selected Tile:")
                .on_hover_text(app.selected_type.explanation());
            draw_tile(&app.selected_type, ui, false);
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            for tile in ALL_TILES {
                if draw_tile(tile, ui, false).clicked() {
                    app.selected_type = tile.clone();
                }
            }
        });
    });
}

fn display_editing_board(ui: &mut egui::Ui, app: &mut App) {
    let mut edited_pos = None;

    // Display the board
    egui::Grid::new("editing_board_grid")
        .spacing(egui::vec2(2.0, 2.0))
        .min_col_width(0.0)
        .show(ui, |ui| {
            for (row_idx, row) in app.editing_model.get_board().iter().enumerate() {
                for (col_idx, tile) in row.iter().enumerate() {
                    let response = draw_tile(tile, ui, false);
                    if response.clicked() {
                        edited_pos = Some((row_idx, col_idx));
                    }
                    if response.hovered() {
                        ui.painter().rect_filled(
                            response.rect,
                            0.0,
                            egui::Color32::from_black_alpha(100),
                        );
                        app.selected_tile_pos = Some((row_idx, col_idx));
                    }
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

    if let Some(edited_pos) = edited_pos {
        app.editing_model
            .set_tile(edited_pos, app.selected_type.clone());
    }
}

/*
    Play mode
*/

fn play_screen(ui: &mut egui::Ui, app: &mut App) {
    ui.label("Playing Mode");
    display_playing_board(ui, app);

    if let Some(mut keypress) = app.get_keys_pressed() {
        if app.playing_model.handle_player_movement(&mut keypress) {
            // Player reached the end tile
            app.mode = AppMode::Editing;
        }
    }
}

fn display_playing_board(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        if ui.button("Switch to Editing Mode").clicked() {
            app.mode = AppMode::Editing;
        }

        ui.add_space(50.0);

        // Display the board using a grid layout
        egui::Grid::new("playing_board_grid")
            .spacing(egui::vec2(2.0, 2.0))
            .min_col_width(0.0)
            .show(ui, |ui| {
                for (row_idx, row) in app.playing_model.get_board().iter().enumerate() {
                    for (col_idx, tile) in row.iter().enumerate() {
                        // Draw faint white border around each cell
                        let rect = draw_tile(
                            tile,
                            ui,
                            (row_idx, col_idx) == app.playing_model.get_player_pos(),
                        )
                        .rect;
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
