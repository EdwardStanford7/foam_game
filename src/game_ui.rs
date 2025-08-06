//!
//! Logic for displaying the game UI and handling user input
//!

const TILE_IMG_SIDE: u32 = 32;
const KEY_IMG_SIDE: u32 = 8;

use super::editing_model::EditingModel;
use super::item::{ALL_KEYS, KeyItem};
use super::playing_model::{MovementPopupData, PlayingModel};
use super::tile::{ALL_TILES, Tile};
use eframe::egui;
use native_dialog::FileDialog;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct KeyState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub space: bool,
    pub enter: bool,
    pub last_update: f64,
    pub keys_pressed_this_frame: bool, // Track if any keys were pressed this frame
}

impl Default for KeyState {
    fn default() -> Self {
        KeyState {
            up: false,
            down: false,
            left: false,
            right: false,
            space: false,
            enter: false,
            last_update: 0.0,
            keys_pressed_this_frame: false,
        }
    }
}

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
    selected_key: KeyItem, // Currently selected key/item for editing
    selected_tile_pos: Option<(usize, usize)>, // Currently selected tile position for editing
    width_slider: usize,   // Width slider for board size
    height_slider: usize,  // Height slider for board size

    key_state: KeyState,
    last_animation_update: f64,

    texture_cache: HashMap<String, egui::TextureHandle>,

    popup_data: Option<PopupData>,
}

#[derive(Debug, Clone)]
pub struct PopupData {
    pub message: String,
    pub popup_type: PopupType,
}

#[derive(Debug, Clone)]
pub enum PopupType {
    Ok,
    YesNo {
        on_yes: fn(&mut App),
        on_no: Option<fn(&mut App)>,
    },
}

// Add method to load image data from file
pub fn load_tile_image(tile: &Tile) -> Result<egui::ColorImage, String> {
    let image = image::ImageReader::open(tile.file_name())
        .map_err(|err| {
            format!(
                "Error loading texture file at {}: {}",
                tile.file_name(),
                err
            )
        })?
        .decode()
        .map_err(|err| format!("Error decoding image at {}: {}", tile.file_name(), err))?;

    // Resize the image to 32x32
    let image = image.resize(
        TILE_IMG_SIDE,
        TILE_IMG_SIDE,
        image::imageops::FilterType::Nearest,
    );
    let size = [TILE_IMG_SIDE as usize, TILE_IMG_SIDE as usize]; // Fixed size
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

pub fn load_key_image(key_item: &KeyItem) -> Result<egui::ColorImage, String> {
    let image = image::ImageReader::open(key_item.file_name())
        .map_err(|err| format!("Error loading key texture file: {err}"))?
        .decode()
        .map_err(|err| format!("Error decoding key image: {err}"))?;

    // Resize the image to 8x8
    let image = image.resize(
        KEY_IMG_SIDE,
        KEY_IMG_SIDE,
        image::imageops::FilterType::Nearest,
    );
    let size = [KEY_IMG_SIDE as usize, KEY_IMG_SIDE as usize]; // Fixed size
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

// Add method to get cached texture
fn load_tile_texture(ctx: &egui::Context, tile: &Tile) -> Result<egui::TextureHandle, String> {
    let image = load_tile_image(tile).map_err(|err| format!("Error loading texture: {err}"))?;

    let texture = ctx.load_texture(tile.file_name(), image, egui::TextureOptions::default());

    Ok(texture)
}

fn load_key_texture(
    ctx: &egui::Context,
    key_item: &KeyItem,
) -> Result<egui::TextureHandle, String> {
    let image =
        load_key_image(key_item).map_err(|err| format!("Error loading key texture: {err}"))?;

    let texture = ctx.load_texture(key_item.file_name(), image, egui::TextureOptions::default());

    Ok(texture)
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut texture_cache = HashMap::new();

        // Pre-load all textures at startup
        for tile in ALL_TILES {
            if let Ok(texture) = load_tile_texture(&cc.egui_ctx, tile) {
                texture_cache.insert(tile.file_name().to_string(), texture);
            } else {
                eprintln!(
                    "Warning: failed to load texture for tile: {}",
                    tile.file_name()
                );
            }
        }

        for key in ALL_KEYS {
            if let Ok(texture) = load_key_texture(&cc.egui_ctx, key) {
                texture_cache.insert(key.file_name().to_string(), texture);
            } else {
                eprintln!(
                    "Warning: failed to load texture for key/item: {}",
                    key.file_name()
                );
            }
        }

        App {
            editing_model: Default::default(),
            playing_model: Default::default(),
            mode: AppMode::Startup,
            selected_type: Tile::Empty,
            selected_key: KeyItem::None,
            selected_tile_pos: None,
            width_slider: 0,
            height_slider: 0,
            texture_cache,
            key_state: KeyState::default(),
            last_animation_update: 0.0,
            popup_data: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints during animation
        if self.playing_model.animation_state.is_some() {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            update_key_state(ui, self);
            match self.mode {
                AppMode::Startup => startup_screen(ui, self),
                AppMode::Editing => editing_screen(ui, self),
                AppMode::Playing => play_screen(ui, self),
            }
        });

        if let Some(PopupData {
            message,
            popup_type,
        }) = self.popup_data.clone()
        {
            egui::Window::new("Result")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(&message);

                    match popup_type {
                        PopupType::Ok => {
                            if ui.button("OK").clicked() {
                                self.popup_data = None;
                            }
                        }
                        PopupType::YesNo { on_yes, on_no } => {
                            if ui.button("Yes").clicked() {
                                on_yes(self);
                                self.popup_data = None;
                            }
                            if ui.button("No").clicked() {
                                if let Some(on_no_fn) = on_no {
                                    on_no_fn(self);
                                }
                                self.popup_data = None;
                            }
                        }
                    }
                });
        }
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
    None,
}

impl DirectionKey {
    // pub fn is_diagonal(&self) -> bool {
    //     matches!(
    //         self,
    //         DirectionKey::UpRight | DirectionKey::DownRight | DirectionKey::DownLeft | DirectionKey::UpLeft
    //     )
    // }
    pub fn is_cardinal(&self) -> bool {
        matches!(
            self,
            DirectionKey::Up | DirectionKey::Right | DirectionKey::Down | DirectionKey::Left
        )
    }
    pub fn is_none(&self) -> bool {
        matches!(self, DirectionKey::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerMovementData {
    pub direction: DirectionKey,
    pub move_speed: usize, // Number of tiles to move in the given direction
    pub use_tile: bool,    // If current tile can be used (e.g. portal)
}

pub fn movement_data_from_bools(
    up: bool,
    right: bool,
    down: bool,
    left: bool,
    move_speed: usize,
    use_tile: bool,
) -> Option<PlayerMovementData> {
    let direction = match (up, right, down, left) {
        (true, false, false, false) => DirectionKey::Up,
        (false, true, false, false) => DirectionKey::Right,
        (false, false, true, false) => DirectionKey::Down,
        (false, false, false, true) => DirectionKey::Left,
        (true, true, false, false) => DirectionKey::UpRight,
        (false, true, true, false) => DirectionKey::DownRight,
        (false, false, true, true) => DirectionKey::DownLeft,
        (true, false, false, true) => DirectionKey::UpLeft,
        _ => DirectionKey::None,
    };

    if direction == DirectionKey::None && !use_tile {
        return None; // No movement or tile usage
    }

    Some(PlayerMovementData {
        direction,
        move_speed,
        use_tile,
    })
}

pub fn direction_key_into_bools(direction: &DirectionKey) -> (bool, bool, bool, bool) {
    let mut up = false;
    let mut right = false;
    let mut down = false;
    let mut left = false;

    match direction {
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
        DirectionKey::None => {}
    }

    (up, right, down, left)
}

impl App {
    pub fn get_movement_data(&mut self) -> Option<PlayerMovementData> {
        if !self.key_state.keys_pressed_this_frame {
            return None;
        }

        let movement_data = movement_data_from_bools(
            self.key_state.up,
            self.key_state.right,
            self.key_state.down,
            self.key_state.left,
            if self.key_state.space { 2 } else { 1 }, // move_speed
            self.key_state.enter,                     // use_tile
        );

        // Clear the key state after consuming it
        self.key_state.up = false;
        self.key_state.down = false;
        self.key_state.left = false;
        self.key_state.right = false;
        self.key_state.space = false;
        self.key_state.enter = false;
        self.key_state.keys_pressed_this_frame = false;

        movement_data
    }
}

fn update_key_state(ui: &mut egui::Ui, app: &mut App) {
    let current_time = ui.input(|i| i.time);
    let mut any_key_pressed = false;
    app.key_state.up = false;
    app.key_state.right = false;
    app.key_state.down = false;
    app.key_state.left = false;
    app.key_state.space = false;

    ui.input(|i| {
        // Check for key presses (not just key down)
        if i.key_pressed(egui::Key::ArrowUp) {
            app.key_state.up = true;
            any_key_pressed = true;
        }
        if i.key_pressed(egui::Key::ArrowDown) {
            app.key_state.down = true;
            any_key_pressed = true;
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            app.key_state.left = true;
            any_key_pressed = true;
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            app.key_state.right = true;
            any_key_pressed = true;
        }
        if i.key_down(egui::Key::Space) {
            app.key_state.space = true;
            any_key_pressed = true;
        }
        if i.key_pressed(egui::Key::Enter) {
            app.key_state.enter = true;
            any_key_pressed = true;
        }

        // Tad hacky but should work. If any key was pressed this frame also check for keys down (to allow multidirectional input)
        if any_key_pressed {
            if i.key_down(egui::Key::ArrowUp) {
                app.key_state.up = true;
            }
            if i.key_down(egui::Key::ArrowDown) {
                app.key_state.down = true;
            }
            if i.key_down(egui::Key::ArrowLeft) {
                app.key_state.left = true;
            }
            if i.key_down(egui::Key::ArrowRight) {
                app.key_state.right = true;
            }
        }
    });

    if any_key_pressed {
        app.key_state.last_update = current_time;
        app.key_state.keys_pressed_this_frame = true;
    } else {
        app.key_state.keys_pressed_this_frame = false;
    }
}

/*
    Draw tile
*/

fn draw_tile_and_key(
    tile: &Tile,
    key: Option<&KeyItem>,
    ui: &mut egui::Ui,
    app: &App,
    player: bool,
) -> (egui::Response, Option<egui::Response>) {
    let (rect, response_tile) =
        ui.allocate_exact_size(egui::Vec2 { x: 32.0, y: 32.0 }, egui::Sense::click());
    let painter = ui.painter_at(rect);

    if let Some(texture) = app.texture_cache.get(tile.file_name()) {
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }

    let response_tile = response_tile.on_hover_text(tile.explanation());

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
            let text = if *val > 0 {
                format!("+{val}")
            } else {
                val.to_string()
            };
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(16.0),
                egui::Color32::RED,
            );
        }
        Tile::Portal(c, _) => {
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

    // Draw key if present
    let response_key = if let Some(key) = key {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2 { x: 8.0, y: 8.0 }, egui::Sense::click());
        let painter = ui.painter_at(rect);
        if key != &KeyItem::None {
            if let Some(texture) = app.texture_cache.get(key.file_name()) {
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }

        // Key overlay
        if let Some(text) = key.overlay() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(16.0),
                egui::Color32::RED,
            );
        }

        let response_key = response.on_hover_text(key.explanation());
        Some(response_key)
    } else {
        None
    };

    if player {
        // Draw player position indicator as a red circle in top right corner
        let circle_radius = 8.0;
        let circle_center = egui::Pos2::new(rect.max.x - circle_radius, rect.min.y + circle_radius);
        painter.circle_filled(circle_center, circle_radius, egui::Color32::BLACK);
    }

    (response_tile, response_key)
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
        } else {
            eprintln!("Error loading board: {}", model.unwrap_err());
        }
    }
}

fn open_file_dialog(is_save: bool) -> Result<String, String> {
    let dialog = FileDialog::new().add_filter("Foam Game Board", &["fg"]);

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

    if let Some(keypress) = app.get_movement_data() {
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

            ui.label("Selected Tile:");
            draw_tile_and_key(&app.selected_type, None, ui, app, false);

            ui.label("Selected Key:");
            draw_tile_and_key(&Tile::Empty, Some(&app.selected_key), ui, app, false);
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            // Tiles
            for tile in ALL_TILES {
                let (tile_response, _) = draw_tile_and_key(tile, None, ui, app, false);
                if tile_response.clicked() {
                    app.selected_type = tile.clone();
                }
                if tile_response.hovered() {
                    ui.painter().rect_filled(
                        tile_response.rect,
                        0.0,
                        egui::Color32::from_black_alpha(100),
                    );
                }
                // white border around each tile
                ui.painter().rect_stroke(
                    tile_response.rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_white_alpha(64)),
                    egui::StrokeKind::Outside,
                );
            }

            // Keys
            for key in ALL_KEYS {
                let (_, key_response) = draw_tile_and_key(&Tile::Empty, Some(key), ui, app, false);
                if let Some(key_response) = key_response {
                    if key_response.clicked() {
                        app.selected_key = key.clone();
                    }
                    if key_response.hovered() {
                        ui.painter().rect_filled(
                            key_response.rect,
                            0.0,
                            egui::Color32::from_black_alpha(100),
                        );
                    }
                    // white border around each key
                    ui.painter().rect_stroke(
                        key_response.rect,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_white_alpha(64)),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        });
    });
}

fn display_editing_board(ui: &mut egui::Ui, app: &mut App) {
    let mut edited_pos = None;

    // Display the board
    egui::Grid::new("editing_board_grid")
        .spacing(egui::vec2(0.0, 0.0))
        .min_col_width(0.0)
        .show(ui, |ui| {
            for (row_idx, row) in app.editing_model.get_board().iter().enumerate() {
                for (col_idx, tile) in row.iter().enumerate() {
                    // Draw each tile and handle clicks
                    let (response_tile, response_key) =
                        draw_tile_and_key(&tile.tile, Some(&tile.key), ui, app, false);
                    if response_tile.clicked() {
                        edited_pos = Some((row_idx, col_idx));
                    }
                    if response_key.map_or(false, |r| r.clicked()) {
                        // TODO: edit key
                    }
                    // Highlight the selected tile
                    if response_tile.hovered() {
                        ui.painter().rect_filled(
                            response_tile.rect,
                            0.0,
                            egui::Color32::from_black_alpha(100),
                        );
                        app.selected_tile_pos = Some((row_idx, col_idx));
                    }
                    let rect = response_tile.rect;
                    // Draw faint white border around each cell
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

const ANIMATION_SPEED: f64 = 0.1; // seconds per tile movement

fn play_screen(ui: &mut egui::Ui, app: &mut App) {
    ui.label("Playing Mode");
    display_playing_board(ui, app);

    if app.playing_model.animation_state.is_none() {
        if let Some(keypress) = app.get_movement_data() {
            app.playing_model.start_movement_animation(keypress);
            app.last_animation_update = ui.input(|i| i.time);
        }
    } else if app.popup_data.is_none() {
        let current_time = ui.input(|i| i.time);
        if current_time - app.last_animation_update > ANIMATION_SPEED {
            app.last_animation_update = current_time;
            match app.playing_model.step_animation(&KeyItem::None) {
                MovementPopupData::None => {}
                MovementPopupData::Wall => {
                    println!("Waiting for wall key");
                    app.popup_data = Some(PopupData {
                        message: "You hit a wall! Do you want to use the red key?".to_string(),
                        popup_type: PopupType::YesNo {
                            on_yes: |_app| {
                                // TODO: update
                                // app.playing_model.step_animation(&KeyItem::OnEquip(
                                //     KeyOnEquip::OnWall(KeyOnWall::Wall),
                                // ));
                            },
                            on_no: Some(|app| {
                                app.playing_model.step_animation(&KeyItem::None);
                            }),
                        },
                    });
                }
                MovementPopupData::Won => {
                    app.popup_data = Some(PopupData {
                        message: "You won! Congratulations!".to_string(),
                        popup_type: PopupType::Ok,
                    });
                    app.mode = AppMode::Editing; // Switch back to editing mode after winning
                }
                MovementPopupData::Lost => {
                    app.popup_data = Some(PopupData {
                        message: "You lost! Better luck next time!".to_string(),
                        popup_type: PopupType::Ok,
                    });
                    app.mode = AppMode::Editing; // Switch back to editing mode after losing
                }
            }
        }
    }
}

fn display_playing_board(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        if ui.button("Switch to Editing Mode").clicked() {
            app.mode = AppMode::Editing;
        }

        ui.add_space(50.0);

        let grid_id = format!(
            "playing_board_grid_{}",
            app.playing_model.get_player_pos().0
        );

        egui::Grid::new(grid_id)
            .spacing(egui::vec2(2.0, 2.0))
            .min_col_width(0.0)
            .show(ui, |ui| {
                for (row_idx, row) in app.playing_model.get_board().iter().enumerate() {
                    for (col_idx, tile) in row.iter().enumerate() {
                        // TODO: do we need to do something with the key response?
                        let (resp, _) = draw_tile_and_key(
                            &tile.tile,
                            Some(&tile.key),
                            ui,
                            app,
                            (row_idx, col_idx) == app.playing_model.get_player_pos(),
                        );
                        let rect = resp.rect;
                        // Draw faint white border around each cell
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
