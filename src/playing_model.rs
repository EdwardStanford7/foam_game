//!
//! Logic for editing and playing the game
//!

use super::tile::Tile;
use crate::{editing_model, game_ui::DirectionKey, game_ui::DirectionKeyWithJump};

#[derive(Debug, Clone, Default)]
pub struct PlayingModel {
    board: Vec<Vec<Tile>>,
    board_size: (usize, usize), // size of the board, including padding
    player_pos: (usize, usize), // position of the player
}

impl PlayingModel {
    pub fn new(editing_model: &editing_model::EditingModel) -> Self {
        let board_size = (
            editing_model.get_board_size().0 + 2,
            editing_model.get_board_size().1 + 2,
        );

        // pad board with layer of empty tiles on outside
        let mut board = vec![vec![Tile::Empty; board_size.0]; board_size.1];
        for (i, row) in editing_model.get_board().iter().enumerate() {
            for (j, tile) in row.iter().enumerate() {
                board[i + 1][j + 1] = tile.clone(); // offset by 1 to account for padding
            }
        }

        let player_pos = (
            editing_model.get_start_pos().unwrap().0 + 1, // offset by 1 to account for padding
            editing_model.get_start_pos().unwrap().1 + 1,
        ); // offset by 1 to account for padding

        PlayingModel {
            board,
            board_size,
            player_pos,
        }
    }

    pub fn get_board(&self) -> &Vec<Vec<Tile>> {
        &self.board
    }

    pub fn get_player_pos(&self) -> (usize, usize) {
        self.player_pos
    }

    // Moves the player and returns true if the game is over
    pub fn handle_player_movement(&mut self, movement: &mut DirectionKeyWithJump) -> bool {
        let mut current_tile = self.board[self.player_pos.0][self.player_pos.1].clone();
        let mut old_pos = self.player_pos;

        println!(
            "Handling player movement: {:?} at position: {:?}",
            movement, self.player_pos
        );

        while !matches!(current_tile, Tile::Empty) {
            match movement.direction {
                DirectionKey::Up => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.0 = self.player_pos.0.saturating_sub(movement.move_speed);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::Right => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.1 =
                            (self.player_pos.1 + movement.move_speed).min(self.board_size.1 - 1);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::Down => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.0 =
                            (self.player_pos.0 + movement.move_speed).min(self.board_size.0 - 1);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::Left => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.1 = self.player_pos.1.saturating_sub(movement.move_speed);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::UpRight => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.0 = self.player_pos.0.saturating_sub(movement.move_speed);
                        self.player_pos.1 = (self.player_pos.1 + 1 + movement.move_speed)
                            .min(self.board_size.1 - 1);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::DownRight => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.0 =
                            (self.player_pos.0 + movement.move_speed).min(self.board_size.0 - 1);
                        self.player_pos.1 =
                            (self.player_pos.1 + movement.move_speed).min(self.board_size.1 - 1);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::DownLeft => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.0 =
                            (self.player_pos.0 + movement.move_speed).min(self.board_size.0 - 1);
                        self.player_pos.1 = self.player_pos.1.saturating_sub(movement.move_speed);
                    } else {
                        break; // Can't move further
                    }
                }
                DirectionKey::UpLeft => {
                    if current_tile.can_move_in_direction(&movement.direction) {
                        self.player_pos.0 = self.player_pos.0.saturating_sub(movement.move_speed);
                        self.player_pos.1 = self.player_pos.1.saturating_sub(movement.move_speed);
                    } else {
                        break; // Can't move further
                    }
                }
            }

            // No movement occurred
            if self.player_pos == old_pos {
                return false;
            }

            // If the current tile is a cloud, remove it
            if matches!(current_tile, Tile::Cloud(_)) {
                self.board[self.player_pos.0][self.player_pos.1] = Tile::Empty;
            }

            // Update the current tile to the new tile
            current_tile = self.board[self.player_pos.0][self.player_pos.1].clone();
            old_pos = self.player_pos;

            match current_tile {
                Tile::EndSpace => {
                    return true; // Player reached the end tile
                }
                Tile::Bounce(amount) => {
                    movement.move_speed =
                        movement.move_speed.checked_add_signed(amount).unwrap_or(0);
                }
                Tile::Ice => movement.move_speed = 1,
                _ => {
                    movement.move_speed = 0; // Reset move speed for non-movement tiles
                }
            }

            println!(
                "Player moved to position: {:?}, current tile: {:?}",
                self.player_pos, current_tile
            );
        }

        true
    }
}
