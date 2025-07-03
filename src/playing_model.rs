//!
//! Logic for editing and playing the game
//!

use crate::editing_model;

use super::tile::{ALL_TILES, Tile};

#[derive(Debug, Clone, Default)]
pub struct PlayingModel {
    board: Vec<Vec<Tile>>,
    player_pos: (usize, usize),          // position of the player
    previous_player_pos: (usize, usize), // previous position of the player for movement logic
    end_pos: (usize, usize),             // position of unique end tile
}

impl PlayingModel {
    pub fn new(editing_model: &editing_model::EditingModel) -> Self {
        let board = editing_model.get_board().clone();
        let player_pos = editing_model.get_start_pos().unwrap(); // Default to (0, 0) if no start position
        let end_pos = editing_model.get_end_pos().unwrap(); // Default to (0, 0) if no end position

        PlayingModel {
            board,
            player_pos,
            previous_player_pos: player_pos,
            end_pos,
        }
    }

    pub fn handle_player_movement(&mut self, recent_keys: &[egui::Key]) {}
}

/*

fn handle_player_movement(ui: &mut egui::Ui, game: &mut App) {
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
            game.mode = GameMode::Editing;
        }

        game.previous_player_pos = game.player_pos; // Store previous position for movement logic
        game.player_pos = (new_pos.0 as usize, new_pos.1 as usize);
    }
}
 */
