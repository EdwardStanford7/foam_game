//!
//! Logic for playing mode - moving around the game board
//!

use super::editing::EditingBoard;
use super::tile::Tile;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayingBoard {
    playing_board: Vec<Vec<Tile>>,
    start_pos: (usize, usize), // position of unique start tile
    end_pos: (usize, usize),   // position of unique end tile
    board_size: (usize, usize), // Size of the board
    player_pos: (usize, usize),
    previous_player_pos: (usize, usize), // Store previous player position for movement logic
}

impl PlayingBoard {
    pub fn new(editing_board: &EditingBoard) -> Option<Self> {
        let playing_board = editing_board.get_board().clone();
        let start_pos = editing_board.get_start_pos()?;
        let board_size = (playing_board.len(), playing_board[0].len());
        let end_pos = editing_board.get_end_pos()?;
        let player_pos = start_pos;
        let previous_player_pos = player_pos;

        Some(Self {
            playing_board,
            start_pos,
            end_pos,
            board_size,
            player_pos,
            previous_player_pos,
        })
    }

    pub fn get_player_position(&self) -> (usize, usize) {
        self.player_pos
    }

    pub fn get_player_position_isize(&self) -> (isize, isize) {
        (self.player_pos.0 as isize, self.player_pos.1 as isize)
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.board_size.0 && y < self.board_size.1 {
            Some(&self.playing_board[x][y])
        } else {
            None
        }
    }

    pub fn current_tile(&self) -> &Tile {
        &self.playing_board[self.player_pos.0][self.player_pos.1]
    }

    pub fn previous_tile(&self) -> &Tile {
        &self.playing_board[self.previous_player_pos.0][self.previous_player_pos.1]
    }

    pub fn position_is_new(&self) -> bool {
        self.player_pos != self.previous_player_pos
    }

    pub fn set_previous_tile(&mut self, tile: Tile) {
        self.playing_board[self.previous_player_pos.0][self.previous_player_pos.1] = tile;
    }

    pub fn pos_is_valid(&self, x: isize, y: isize) -> bool {
        x < self.board_size.0 as isize && y < self.board_size.1 as isize
    }

    pub fn pos_is_end_square(&self, x: isize, y: isize) -> bool {
        (x, y) == (self.end_pos.0 as isize, self.end_pos.1 as isize)
    }

    pub fn advance_player_position(&mut self, new_pos: (isize, isize)) {
        self.previous_player_pos = self.player_pos;
        self.player_pos = (new_pos.0 as usize, new_pos.1 as usize);
    }
}
