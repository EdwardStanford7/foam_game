//!
//! Logic for editing mode - editing the game board
//!

use super::tile::Tile;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EditingBoard {
    selected_type: Tile,
    selected_tile_pos: Option<(usize, usize)>, // Currently selected tile position for editing
    editing_board: Vec<Vec<Tile>>,
    start_pos: Option<(usize, usize)>, // position of unique start tile
    end_pos: Option<(usize, usize)>,   // position of unique end tile
}

impl EditingBoard {
    fn has_start(&self) -> bool {
        self.editing_board.iter().any(|row| row.contains(&Tile::StartSpace))
    }

    fn has_end(&self) -> bool {
        self.editing_board.iter().any(|row| row.contains(&Tile::EndSpace))
    }

    pub fn is_playable_board(&self) -> bool {
        // Check if the board has a start and end tile
        self.has_start() && self.has_end()

        // todo later check things like matching portal pairs, etc.
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.editing_board = vec![vec![Tile::Empty; width]; height];
        self.start_pos = None;
        self.end_pos = None;
    }

    pub fn select_tile_position(&mut self, x: usize, y: usize) {
        if x < self.editing_board.len() && y < self.editing_board[x].len() {
            self.selected_tile_pos = Some((x, y));
        } else {
            // TODO?
            eprintln!("Selected tile position out of bounds: ({}, {})", x, y);
            self.selected_tile_pos = None; // Reset if out of bounds
        }
    }

    pub fn select_type(&mut self, tile: Tile) {
        self.selected_type = tile;
    }

    pub fn has_selected_tile(&self) -> bool {
        self.selected_tile_pos.is_some()
    }

    pub fn get_selected_type(&self) -> &Tile {
        &self.selected_type
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.editing_board.get(x)?.get(y)
    }

    pub fn get_selected_tile(&self) -> Option<&Tile> {
        let (x, y) = self.selected_tile_pos?;
        self.get_tile(x, y)
    }

    pub fn get_start_pos(&self) -> Option<(usize, usize)> {
        self.start_pos
    }

    pub fn set_start_pos(&mut self, x: usize, y: usize) {
        self.start_pos = Some((x, y));
    }

    pub fn get_end_pos(&self) -> Option<(usize, usize)> {
        self.end_pos
    }

    pub fn set_end_pos(&mut self, x: usize, y: usize) {
        self.end_pos = Some((x, y));
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.editing_board.get_mut(x)?.get_mut(y)
    }

    pub fn get_selected_tile_mut(&mut self) -> Option<&mut Tile> {
        let (x, y) = self.selected_tile_pos?;
        self.get_tile_mut(x, y)
    }

    pub fn get_board(&self) -> &Vec<Vec<Tile>> {
        &self.editing_board
    }
}

impl Default for EditingBoard {
    fn default() -> Self {
        Self {
            selected_type: Tile::Empty,
            selected_tile_pos: None,
            editing_board: vec![],
            start_pos: None,
            end_pos: None,
        }
    }
}

/*
    GUI
*/
