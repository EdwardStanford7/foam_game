//!
//! Game model for keys (single-use items).
//!

use serde::{Deserialize, Serialize};

/// Keys that activate on receiving them
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnGet {
    FinishKey, // Must get before going to finish
}

/// Keys that activate on use
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnUse {
    TeleportKey(char), // Teleport to a portal
}

/// Keys that activate on movement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnMovement {
    Cardinal, // Move in a (disallowed) cardinal direction
    Diagonal, // Move in a diagonal direction
}

/// Keys that activate on hitting a wall
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnWall {
    DoorKey(char), // Open a door
    Wall,          // Jump over a wall
}

/// Keys that activate mid-bounce
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnBounce {
    BounceLess,   // Bounce -1 less
    BounceMore,   // Bounce +1 more
    BounceChange, // Change bounce direction
}

/// Keys that activate on landing on an empty tile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnEmpty {
    CloudKey, // Jump on air
}

/// Keys that are equiped
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyOnEquip {
    OnMovement(KeyOnMovement),
    OnWall(KeyOnWall),
    OnBounce(KeyOnBounce),
    OnEmpty(KeyOnEmpty),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeyItem {
    None, // No key item
    OnGet(KeyOnGet),
    OnUse(KeyOnUse),
    OnEquip(KeyOnEquip),
}

use KeyOnBounce::*;
use KeyOnEmpty::*;
use KeyOnEquip::*;
use KeyOnGet::*;
use KeyOnMovement::*;
use KeyOnUse::*;
use KeyOnWall::*;

impl KeyItem {
    pub fn file_name(&self) -> &str {
        match self {
            KeyItem::None => "assets/keys/none.png",
            KeyItem::OnGet(FinishKey) => "assets/keys/finish.png",
            KeyItem::OnUse(TeleportKey(_c)) => "assets/keys/teleport.png",
            KeyItem::OnEquip(OnMovement(Cardinal)) => "assets/keys/cardinal.png",
            KeyItem::OnEquip(OnMovement(Diagonal)) => "assets/keys/diagonal.png",
            KeyItem::OnEquip(OnWall(DoorKey(_c))) => "assets/keys/door.png",
            KeyItem::OnEquip(OnWall(Wall)) => "assets/keys/wall.png",
            KeyItem::OnEquip(OnBounce(BounceLess)) => "assets/keys/bounce_less.png",
            KeyItem::OnEquip(OnBounce(BounceMore)) => "assets/keys/bounce_more.png",
            KeyItem::OnEquip(OnBounce(BounceChange)) => "assets/keys/bounce_change.png",
            KeyItem::OnEquip(OnEmpty(CloudKey)) => "assets/keys/cloud.png",
        }
    }

    /// Overlay symbol to draw over the key, if any
    pub fn overlay(&self) -> Option<char> {
        match self {
            &KeyItem::OnUse(TeleportKey(c)) => Some(c),
            &KeyItem::OnEquip(OnWall(DoorKey(c))) => Some(c),
            _ => None,
        }
    }

    pub fn explanation(&self) -> &str {
        match self {
            KeyItem::None => "No key item.",
            KeyItem::OnGet(FinishKey) => "A key that must be collected before reaching the end.",
            KeyItem::OnUse(TeleportKey(_c)) => {
                "A key that teleports you to a portal with the same letter."
            }
            KeyItem::OnEquip(OnMovement(Cardinal)) => {
                "A key that allows you to move in a disallowed cardinal direction."
            }
            KeyItem::OnEquip(OnMovement(Diagonal)) => {
                "A key that allows you to move in a disallowed diagonal direction."
            }
            KeyItem::OnEquip(OnWall(DoorKey(_c))) => {
                "A key that opens a door with the same letter."
            }
            KeyItem::OnEquip(OnWall(Wall)) => "A key that allows you to jump over walls.",
            KeyItem::OnEquip(OnBounce(BounceLess)) => "A key that reduces your bounce by 1.",
            KeyItem::OnEquip(OnBounce(BounceMore)) => "A key that increases your bounce by 1.",
            KeyItem::OnEquip(OnBounce(BounceChange)) => "A key that changes your bounce direction.",
            KeyItem::OnEquip(OnEmpty(CloudKey)) => "A key that allows you to jump on empty tiles.",
        }
    }
}

pub const ALL_KEYS: &[KeyItem] = &[
    KeyItem::OnGet(FinishKey),
    KeyItem::OnUse(TeleportKey('A')),
    KeyItem::OnEquip(OnMovement(Cardinal)),
    KeyItem::OnEquip(OnMovement(Diagonal)),
    KeyItem::OnEquip(OnWall(DoorKey('A'))),
    KeyItem::OnEquip(OnWall(Wall)),
    KeyItem::OnEquip(OnBounce(BounceLess)),
    KeyItem::OnEquip(OnBounce(BounceMore)),
    KeyItem::OnEquip(OnBounce(BounceChange)),
    KeyItem::OnEquip(OnEmpty(CloudKey)),
];
