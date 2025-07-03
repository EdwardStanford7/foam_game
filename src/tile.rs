//!
//! Game board tiles.
//!

use crate::game_ui::DirectionKey;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct CardinalDirectionsAllowed {
    pub up: bool,
    pub right: bool,
    pub down: bool,
    pub left: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct DiagonalDirectionsAllowed {
    pub up_right: bool,
    pub down_right: bool,
    pub down_left: bool,
    pub up_left: bool,
}

// Each tile occupies one space on the board, and has different rules for movement
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum Tile {
    Empty,
    MoveCardinal(CardinalDirectionsAllowed),
    MoveDiagonal(DiagonalDirectionsAllowed), // Move in specific directions, can be cardinal or diagonal
    Cloud(CardinalDirectionsAllowed),        // Clouds, disappear after one use
    Bounce(isize), // Bounce some amount of squares, +/- some amount of acceleration or deceleration
    Portal(char),  // Portal, teleport to other portal with same letter
    Water,         // Water
    Ice,           // Ice
    Door,          // Doors, requires
    Wall,          // Blocks movement
    StartSpace,    // Start space, where the player starts
    EndSpace,      // End space, puzzle completion
}

pub const ALL_TILES: &[Tile] = &[
    Tile::MoveCardinal(CardinalDirectionsAllowed {
        up: true,
        right: true,
        down: true,
        left: true,
    }),
    Tile::MoveDiagonal(DiagonalDirectionsAllowed {
        up_right: true,
        down_right: true,
        down_left: true,
        up_left: true,
    }),
    Tile::Cloud(CardinalDirectionsAllowed {
        up: true,
        right: true,
        down: true,
        left: true,
    }),
    Tile::Bounce(0),
    Tile::Portal('A'),
    Tile::Water,
    Tile::Ice,
    Tile::Door,
    Tile::Wall,
    Tile::StartSpace,
    Tile::EndSpace,
    Tile::Empty,
];

impl Tile {
    pub fn file_name(&self) -> &str {
        match self {
            Tile::Empty => "assets/empty.png",
            Tile::MoveCardinal(_) => "assets/move_cardinal.png",
            Tile::MoveDiagonal(_) => "assets/move_diagonal.png",
            Tile::Cloud(_) => "assets/cloud.png",
            Tile::Bounce(_) => "assets/bounce.png",
            Tile::Portal(_) => "assets/portal.png",
            Tile::Water => "assets/water.png",
            Tile::Ice => "assets/ice.png",
            Tile::Door => "assets/door.png",
            Tile::Wall => "assets/wall.png",
            Tile::StartSpace => "assets/start_space.png",
            Tile::EndSpace => "assets/end_space.png",
        }
    }

    pub fn explanation(&self) -> &str {
        match self {
            Tile::Empty => "An empty tile, no special properties.",
            Tile::MoveCardinal(_) => {
                "A tile that allows moving up, down, left, right. Use arrow keys to toggle directions."
            }
            Tile::MoveDiagonal(_) => {
                "A tile that allows moving up-right, down-right, down-left, up-left. Use arrow keys to toggle directions."
            }
            Tile::Cloud(_) => {
                "A cloud tile that disappears after one use. Use arrow keys to toggle directions."
            }
            Tile::Bounce(_) => {
                "A tile that bounces the player a certain distance. Use up and down to set the bounce modifier."
            }
            Tile::Portal(_) => {
                "A portal tile that teleports the player to another location. Type a letter to identify the portal."
            }
            Tile::Door => {
                "A door tile, which requires a key to pass. Type a letter to identify the door."
            }
            Tile::Water => "A water tile, which we made up and we have no idea what it does KEKW.",
            Tile::Ice => "An ice tile, which causes the player to slide.",
            Tile::Wall => "A wall tile, which blocks movement.",
            Tile::StartSpace => "The starting space for the player.",
            Tile::EndSpace => "The end space for the puzzle completion.",
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
                !(!directions.up && !directions.down && !directions.left && !directions.right)
            }
            Tile::MoveDiagonal(directions) => {
                !(!directions.up_right
                    && !directions.down_right
                    && !directions.down_left
                    && !directions.up_left)
            }
            Tile::Empty
            | Tile::Bounce(_)
            | Tile::Portal(_)
            | Tile::Water
            | Tile::Ice
            | Tile::Door
            | Tile::Wall
            | Tile::StartSpace
            | Tile::EndSpace => true,
        }
    }

    pub fn can_move_in_direction(&self, direction: &DirectionKey) -> bool {
        match self {
            Tile::MoveCardinal(directions) => match direction {
                DirectionKey::Up => directions.up,
                DirectionKey::Right => directions.right,
                DirectionKey::Down => directions.down,
                DirectionKey::Left => directions.left,
                _ => false,
            },
            Tile::Cloud(directions) => match direction {
                DirectionKey::Up => directions.up,
                DirectionKey::Right => directions.right,
                DirectionKey::Down => directions.down,
                DirectionKey::Left => directions.left,
                _ => false,
            },
            Tile::MoveDiagonal(directions) => match direction {
                DirectionKey::UpRight => directions.up_right,
                DirectionKey::DownRight => directions.down_right,
                DirectionKey::DownLeft => directions.down_left,
                DirectionKey::UpLeft => directions.up_left,
                _ => false,
            },
            _ => matches!(
                direction,
                DirectionKey::Up | DirectionKey::Right | DirectionKey::Down | DirectionKey::Left
            ), // Other tiles allow movement in any cardinal direction
        }
    }
}
