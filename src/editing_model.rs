use super::tile::Tile;
use serde::{Deserialize, Serialize};

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
        let board_data = serde_json::to_string(&self.board)
            .map_err(|err| format!("Error serializing board data: {}", err))?;
        std::fs::write(file, board_data)
            .map_err(|err| format!("Error writing board file: {}", err))?;
        Ok(())
    }

    pub fn board_is_playable(&self) -> bool {
        self.start_pos.is_some() && self.end_pos.is_some() // TODO: teleports, doors, keys
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

    pub fn get_end_pos(&self) -> Option<(usize, usize)> {
        self.end_pos
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

    pub fn edit_tile(&mut self, pos: (usize, usize), keys: &[egui::Key]) {
        if let Some(tile) = self.board.get_mut(pos.0).and_then(|row| row.get_mut(pos.1)) {
            match tile {
                Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
                    for (key, direction) in [
                        (egui::Key::ArrowUp, &mut directions.up),
                        (egui::Key::ArrowDown, &mut directions.down),
                        (egui::Key::ArrowLeft, &mut directions.left),
                        (egui::Key::ArrowRight, &mut directions.right),
                    ] {
                        if keys.contains(&key) {
                            *direction = !*direction;
                        }
                    }
                }
                Tile::MoveDiagonal(dirs) => {
                    let diagonals = [
                        (
                            (egui::Key::ArrowUp, egui::Key::ArrowRight),
                            &mut dirs.up_right,
                        ),
                        (
                            (egui::Key::ArrowRight, egui::Key::ArrowDown),
                            &mut dirs.down_right,
                        ),
                        (
                            (egui::Key::ArrowDown, egui::Key::ArrowLeft),
                            &mut dirs.down_left,
                        ),
                        (
                            (egui::Key::ArrowLeft, egui::Key::ArrowUp),
                            &mut dirs.up_left,
                        ),
                    ];
                    for ((k1, k2), dir) in diagonals {
                        if keys.contains(&k1) && keys.contains(&k2) {
                            *dir = !*dir;
                        }
                    }
                }
                Tile::Bounce(val) => {
                    if keys.contains(&egui::Key::ArrowUp) && *val < 1 {
                        *val += 1;
                    } else if keys.contains(&egui::Key::ArrowDown) && *val > -1 {
                        *val -= 1;
                    }
                }
                Tile::Portal(c) => {
                    if keys.contains(&egui::Key::ArrowUp) {
                        *c = match *c {
                            'A'..='Y' => (*c as u8 + 1) as char,
                            'Z' => 'A',
                            _ => 'A',
                        };
                    } else if keys.contains(&egui::Key::ArrowDown) {
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
