//!
//! Logic for editing and playing the game
//!

use super::tile::Tile;
use crate::{editing_model, game_ui::DirectionKey, game_ui::PlayerMovementData};

#[derive(Debug, Clone)]
pub struct PlayingAnimationState {
    pub current_tile: Tile,
    pub old_pos: (usize, usize), // previous position of the player
    pub movement_speed: usize,
    pub direction: DirectionKey,
    pub use_tile: bool,
    pub finished: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PlayingModel {
    board: Vec<Vec<Tile>>,
    board_size: (usize, usize), // size of the board, including padding
    player_pos: (usize, usize), // position of the player
    pub animation_state: Option<PlayingAnimationState>,
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
            editing_model.get_start_pos().unwrap().1 + 1, // offset by 1 to account for padding
        );

        PlayingModel {
            board,
            board_size,
            player_pos,
            animation_state: None,
        }
    }

    pub fn get_board(&self) -> &Vec<Vec<Tile>> {
        &self.board
    }

    pub fn get_player_pos(&self) -> (usize, usize) {
        self.player_pos
    }

    pub fn start_movement_animation(&mut self, movement: PlayerMovementData) {
        if !self.board[self.player_pos.0][self.player_pos.1]
            .can_move_in_direction(&movement.direction)
        {
            self.animation_state = None;
            return;
        }

        self.animation_state = Some(PlayingAnimationState {
            current_tile: self.board[self.player_pos.0][self.player_pos.1].clone(),
            old_pos: self.player_pos,
            movement_speed: movement.move_speed,
            direction: movement.direction,
            use_tile: movement.use_tile,
            finished: false,
        });
    }

    pub fn step_animation(&mut self) -> bool {
        if let Some(state) = &mut self.animation_state {
            if state.finished {
                self.animation_state = None;
                return false;
            }

            state.current_tile = self.board[self.player_pos.0][self.player_pos.1].clone();
            state.old_pos = self.player_pos;

            match state.direction {
                DirectionKey::Up => {
                    self.player_pos.0 = self.player_pos.0.saturating_sub(state.movement_speed)
                }
                DirectionKey::Down => {
                    self.player_pos.0 = (self.player_pos.0 + state.movement_speed)
                        .min(self.board_size.0 - state.movement_speed)
                }
                DirectionKey::Left => {
                    self.player_pos.1 = self.player_pos.1.saturating_sub(state.movement_speed)
                }
                DirectionKey::Right => {
                    self.player_pos.1 = (self.player_pos.1 + state.movement_speed)
                        .min(self.board_size.1 - state.movement_speed);
                }
                DirectionKey::UpLeft => {
                    self.player_pos.0 = self.player_pos.0.saturating_sub(state.movement_speed);
                    self.player_pos.1 = self.player_pos.1.saturating_sub(state.movement_speed);
                }
                DirectionKey::UpRight => {
                    self.player_pos.0 = self.player_pos.0.saturating_sub(state.movement_speed);
                    self.player_pos.1 = (self.player_pos.1 + state.movement_speed)
                        .min(self.board_size.1 - state.movement_speed);
                }
                DirectionKey::DownLeft => {
                    self.player_pos.0 = (self.player_pos.0 + state.movement_speed)
                        .min(self.board_size.0 - state.movement_speed);
                    self.player_pos.1 = self.player_pos.1.saturating_sub(state.movement_speed);
                }
                DirectionKey::DownRight => {
                    self.player_pos.0 = (self.player_pos.0 + state.movement_speed)
                        .min(self.board_size.0 - state.movement_speed);
                    self.player_pos.1 = (self.player_pos.1 + state.movement_speed)
                        .min(self.board_size.1 - state.movement_speed);
                }
                DirectionKey::None => {
                    if let Tile::Portal(_, pos) = state.current_tile {
                        if state.use_tile {
                            self.player_pos.0 = pos.0 + 1; // offset by 1 to account for padding
                            self.player_pos.1 = pos.1 + 1; // offset by 1 to account for padding
                        }
                    }
                    state.finished = true;
                    return false;
                }
            }

            // No movement occurred
            if self.player_pos == state.old_pos {
                state.finished = true;
                return false;
            }

            // If the current tile is a cloud, remove it
            if matches!(state.current_tile, Tile::Cloud(_)) {
                self.board[self.player_pos.0][self.player_pos.1] = Tile::Empty;
            }

            // Check if there is a wall in between the old position and the new position
            let start_row = state.old_pos.0.min(self.player_pos.0);
            let end_row = state.old_pos.0.max(self.player_pos.0);
            let start_col = state.old_pos.1.min(self.player_pos.1);
            let end_col = state.old_pos.1.max(self.player_pos.1);

            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    if self.board[row][col] == Tile::Wall {
                        // If there is a wall, revert to the position right in front of the wall
                        self.player_pos = if state.old_pos.0 < self.player_pos.0 {
                            (row.saturating_sub(1), col) // Move up
                        } else if state.old_pos.0 > self.player_pos.0 {
                            (row + 1, col) // Move down
                        } else if state.old_pos.1 < self.player_pos.1 {
                            (row, col.saturating_sub(1)) // Move left
                        } else {
                            (row, col + 1) // Move right
                        };
                        return false; // Can't move further
                    }
                }
            }

            // Apply movement
            state.current_tile = self.board[self.player_pos.0][self.player_pos.1].clone();
            state.old_pos = self.player_pos;

            match state.current_tile {
                Tile::EndSpace => {
                    state.finished = true;
                    return true; // End game
                }
                Tile::Bounce(amount) => {
                    state.movement_speed =
                        state.movement_speed.checked_add_signed(amount).unwrap_or(0);
                }
                Tile::Ice => {
                    state.movement_speed = 1;
                }
                Tile::Empty => {
                    return true; // End game
                }
                _ => {
                    state.movement_speed = 0;
                }
            }

            if state.movement_speed == 0 {
                state.finished = true;
            }
        }

        false
    }
}
