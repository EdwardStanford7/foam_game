use super::tile::Tile;
use serde::{Deserialize, Serialize};

use super::game_ui::{self, DirectionKeyWithJump};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditingModel {
    board: Vec<Vec<Tile>>,             // rows then columns
    board_size: (usize, usize),        // size of the board (width, height)
    start_pos: Option<(usize, usize)>, // position of unique start tile
    end_pos: Option<(usize, usize)>,   // position of unique end tile
}

impl EditingModel {
    pub fn new(board_size: (usize, usize)) -> Self {
        let board = vec![vec![Tile::Empty; board_size.0]; board_size.1]; // Rows (x) then columns (y)
        EditingModel {
            board,
            board_size,
            start_pos: None,
            end_pos: None,
        }
    }

    pub fn load_board(file: &str) -> Result<Self, String> {
        let model_raw = std::fs::read_to_string(file)
            .map_err(|err| format!("Error reading board file: {}", err))?;
        let model: EditingModel = serde_json::from_str(&model_raw)
            .map_err(|err| format!("Error deserializing board data: {}", err))?;
        Ok(model)
    }

    pub fn save_board(&self, file: &str) -> Result<(), String> {
        let model_data = serde_json::to_string(&self)
            .map_err(|err| format!("Error serializing board data: {}", err))?;
        std::fs::write(file, model_data)
            .map_err(|err| format!("Error writing board file: {}", err))?;
        Ok(())
    }

    pub fn board_is_playable(&self) -> bool {
        if !(self.start_pos.is_some() && self.end_pos.is_some()) {
            return false; // TODO: teleports, doors, keys
        }

        for tile in self.board.iter().flatten() {
            if !tile.is_valid() {
                return false; // Invalid tile found
            }
        }

        true
    }

    pub fn get_board_size(&self) -> (usize, usize) {
        self.board_size
    }

    pub fn get_board(&self) -> &Vec<Vec<Tile>> {
        &self.board
    }

    pub fn get_start_pos(&self) -> Option<(usize, usize)> {
        self.start_pos
    }

    pub fn set_tile(&mut self, pos: (usize, usize), tile: Tile) {
        if matches!(tile, Tile::StartSpace) {
            if let Some(old) = self.start_pos.take() {
                self.board[old.0][old.1] = Tile::Empty; // Remove old start tile
            }
            self.start_pos = Some(pos);
        } else if matches!(tile, Tile::EndSpace) {
            if let Some(old) = self.end_pos.take() {
                self.board[old.0][old.1] = Tile::Empty; // Remove old end tile
            }
            self.end_pos = Some(pos);
        }

        self.board[pos.0][pos.1] = tile;
    }

    pub fn edit_tile(&mut self, pos: (usize, usize), keypress: &DirectionKeyWithJump) {
        let (key_up, key_right, key_down, key_left, _) =
            game_ui::direction_key_into_bools(keypress);
        if let Some(tile) = self.board.get_mut(pos.0).and_then(|row| row.get_mut(pos.1)) {
            match tile {
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
                    let test_tile = match tile {
                        Tile::MoveCardinal(_) => Tile::MoveCardinal(new_directions.clone()),
                        Tile::Cloud(_) => Tile::Cloud(new_directions.clone()),
                        _ => unreachable!(),
                    };
                    if test_tile.is_valid() {
                        *tile = test_tile;
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
                            *tile = test_tile;
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
                Tile::Portal(c) => {
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
