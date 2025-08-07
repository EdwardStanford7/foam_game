use super::game_ui::{self, PlayerMovementData};
use super::item::KeyItem;
use super::tile::{Tile, TileData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditingModel {
    board: Vec<Vec<TileData>>,         // rows then columns
    board_size: (usize, usize),        // size of the board (width, height)
    start_pos: Option<(usize, usize)>, // position of unique start tile
    end_pos: Option<(usize, usize)>,   // position of unique end tile
}

impl EditingModel {
    pub fn new(board_size: (usize, usize)) -> Self {
        let board = vec![vec![TileData::empty(); board_size.1]; board_size.0]; // Rows (x) then columns (y)
        EditingModel {
            board,
            board_size,
            start_pos: None,
            end_pos: None,
        }
    }

    pub fn load_board(file: &str) -> Result<Self, String> {
        let model_raw = std::fs::read_to_string(file)
            .map_err(|err| format!("Error reading board file: {err}"))?;
        let model: EditingModel = serde_json::from_str(&model_raw)
            .map_err(|err| format!("Error deserializing board data: {err}"))?;
        Ok(model)
    }

    pub fn save_board(&self, file: &str) -> Result<(), String> {
        let model_data = serde_json::to_string(&self)
            .map_err(|err| format!("Error serializing board data: {err}"))?;
        std::fs::write(file, model_data)
            .map_err(|err| format!("Error writing board file: {err}"))?;
        Ok(())
    }

    pub fn board_is_playable(&mut self) -> bool {
        if !(self.start_pos.is_some() && self.end_pos.is_some()) {
            return false;
        }

        let mut portal_positions = std::collections::HashMap::<char, Vec<(usize, usize)>>::new();

        for (row_idx, row) in self.board.iter().enumerate() {
            for (col_idx, tile) in row.iter().enumerate() {
                let TileData { tile, key: _ } = &tile;

                if !tile.is_valid() {
                    return false; // Invalid tile found
                }

                if let Tile::Portal(c, _) = tile {
                    portal_positions
                        .entry(*c)
                        .or_default()
                        .push((row_idx, col_idx));
                }
            }

            // TODO: verify that keys are valid
            // The only important thing here is probably that the teleport/door keys have corresponding tiles
        }

        // Check that all portal letters appear exactly twice
        for (_, positions) in portal_positions.iter() {
            if positions.len() != 2 {
                return false; // Portal letter appears more or less than twice
            }
        }

        // Verify that portals are properly linked to each other
        for (letter, positions) in portal_positions.iter() {
            self.board[positions[0].0][positions[0].1].tile = Tile::Portal(*letter, positions[1]); // Link first portal to second
            self.board[positions[1].0][positions[1].1].tile = Tile::Portal(*letter, positions[0]); // Link second portal to first
        }

        true
    }

    pub fn get_board_size(&self) -> (usize, usize) {
        self.board_size
    }

    pub fn get_board(&self) -> &Vec<Vec<TileData>> {
        &self.board
    }

    pub fn get_start_pos(&self) -> Option<(usize, usize)> {
        self.start_pos
    }

    pub fn set_tile(&mut self, pos: (usize, usize), tile: Tile) {
        if matches!(tile, Tile::StartSpace) {
            if let Some(old) = self.start_pos.take() {
                self.board[old.0][old.1].tile = Tile::Empty; // Remove old start tile
            }
            self.start_pos = Some(pos);
        } else if matches!(tile, Tile::EndSpace) {
            if let Some(old) = self.end_pos.take() {
                self.board[old.0][old.1].tile = Tile::Empty; // Remove old end tile
            }
            self.end_pos = Some(pos);
        }

        self.board[pos.0][pos.1].tile = tile;
    }

    pub fn set_key(&mut self, pos: (usize, usize), key: KeyItem) {
        if let Some(tile_data) = self.board.get_mut(pos.0).and_then(|row| row.get_mut(pos.1))
            && tile_data.tile != Tile::Empty
        {
            tile_data.key = key;
        }
    }

    pub fn edit_tile(&mut self, pos: (usize, usize), keypress: &PlayerMovementData) {
        let (key_up, key_right, key_down, key_left) =
            game_ui::direction_key_into_bools(&keypress.direction);
        if let Some(tile_data) = self.board.get_mut(pos.0).and_then(|row| row.get_mut(pos.1)) {
            match &mut tile_data.tile {
                Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
                    let mut new_directions = directions.clone();
                    for (key_pressed, direction) in [
                        (key_up, &mut new_directions.up),
                        (key_down, &mut new_directions.down),
                        (key_left, &mut new_directions.left),
                        (key_right, &mut new_directions.right),
                    ] {
                        if key_pressed {
                            *direction = !*direction;
                        }
                    }
                    let test_tile = match &tile_data.tile {
                        Tile::MoveCardinal(_) => Tile::MoveCardinal(new_directions.clone()),
                        Tile::Cloud(_) => Tile::Cloud(new_directions.clone()),
                        _ => unreachable!(),
                    };
                    if test_tile.is_valid() {
                        tile_data.tile = test_tile;
                    }
                }
                Tile::MoveDiagonal(dirs) => {
                    let mut new_dirs = dirs.clone();
                    let diagonal = if key_up && key_right {
                        Some(&mut new_dirs.up_right)
                    } else if key_down && key_right {
                        Some(&mut new_dirs.down_right)
                    } else if key_down && key_left {
                        Some(&mut new_dirs.down_left)
                    } else if key_up && key_left {
                        Some(&mut new_dirs.up_left)
                    } else {
                        None
                    };
                    if let Some(dir) = diagonal {
                        *dir = !*dir;
                        let test_tile = Tile::MoveDiagonal(new_dirs.clone());
                        if test_tile.is_valid() {
                            tile_data.tile = test_tile;
                        }
                    }
                }
                Tile::Bounce(val) => {
                    if key_up && *val < 1 {
                        *val += 1;
                    } else if key_down && *val > -1 {
                        *val -= 1;
                    }
                }
                Tile::Portal(c, _) => {
                    if key_up {
                        *c = match *c {
                            'A'..='Y' => (*c as u8 + 1) as char,
                            'Z' => 'A',
                            _ => 'A',
                        };
                    } else if key_down {
                        *c = match *c {
                            'B'..='Z' => (*c as u8 - 1) as char,
                            'A' => 'Z',
                            _ => 'Z',
                        };
                    }
                }
                _ => {}
            }
        }
    }
}
