//!
//! Game board tiles.
//!

use super::game_ui::DirectionKey;
use super::item::KeyItem;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CardinalDirectionsAllowed {
    pub up: bool,
    pub right: bool,
    pub down: bool,
    pub left: bool,
}

impl CardinalDirectionsAllowed {
    pub fn allows(&self, direction: &DirectionKey) -> bool {
        match direction {
            DirectionKey::Up => self.up,
            DirectionKey::Right => self.right,
            DirectionKey::Down => self.down,
            DirectionKey::Left => self.left,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DiagonalDirectionsAllowed {
    pub up_right: bool,
    pub down_right: bool,
    pub down_left: bool,
    pub up_left: bool,
}

impl DiagonalDirectionsAllowed {
    pub fn allows(&self, direction: &DirectionKey) -> bool {
        match direction {
            DirectionKey::UpRight => self.up_right,
            DirectionKey::DownRight => self.down_right,
            DirectionKey::DownLeft => self.down_left,
            DirectionKey::UpLeft => self.up_left,
            _ => false,
        }
    }
}

// Each tile occupies one space on the board, and has different rules for movement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Tile {
    Empty,
    MoveCardinal(CardinalDirectionsAllowed),
    MoveDiagonal(DiagonalDirectionsAllowed), // Move in specific directions, can be cardinal or diagonal
    Cloud(CardinalDirectionsAllowed),        // Clouds, disappear after one use
    Bounce(isize), // Bounce some amount of squares, +/- some amount of acceleration or deceleration
    Portal(char, (usize, usize)), // Portal, teleport to other portal with same letter
    Ice,           // Ice
    Door,          // Doors
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
    Tile::Portal('A', (0, 0)),
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
            Tile::Portal(..) => "assets/portal.png",
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
            Tile::Portal(..) => {
                "A portal tile that teleports the player to another location. Type a letter to identify the portal."
            }
            Tile::Door => {
                "A door tile, which requires a key to pass. Type a letter to identify the door."
            }
            Tile::Ice => "An ice tile, which causes the player to slide.",
            Tile::Wall => "A wall tile, which blocks movement.",
            Tile::StartSpace => "The starting space for the player.",
            Tile::EndSpace => "The end space for the puzzle completion.",
        }
    }

    /// Check if the tile is valid for the game rules - if not, will block playing
    pub fn is_valid(&self) -> bool {
        match self {
            Tile::MoveCardinal(directions) | Tile::Cloud(directions) => {
                directions.up || directions.down || directions.left || directions.right
            }
            Tile::MoveDiagonal(directions) => {
                directions.up_right
                    || directions.down_right
                    || directions.down_left
                    || directions.up_left
            }
            &Tile::Bounce(u) => (-1..=1).contains(&u),
            Tile::Empty
            | Tile::Portal(..)
            | Tile::Ice
            | Tile::Door
            | Tile::Wall
            | Tile::StartSpace
            | Tile::EndSpace => true,
        }
    }

    pub fn can_move_in_direction(&self, direction: &DirectionKey) -> bool {
        match self {
            Tile::MoveCardinal(directions) => directions.allows(direction),
            Tile::Cloud(directions) => directions.allows(direction),
            Tile::MoveDiagonal(directions) => directions.allows(direction),
            Tile::Portal(..) => direction.is_cardinal() || direction.is_none(),
            _ => direction.is_cardinal(),
        }
    }
}

/*
    TileData struct - title with associated item
*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TileData {
    pub tile: Tile,
    // TBD: could be a vec of items later
    pub key: KeyItem,
}

impl TileData {
    pub fn empty() -> Self {
        TileData {
            tile: Tile::Empty,
            key: KeyItem::None,
        }
    }
}

impl Default for TileData {
    fn default() -> Self {
        TileData::empty()
    }
}
