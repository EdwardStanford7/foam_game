//!
//! Game model for keys (single-use items).
//!

#[derive(Debug, Clone)]
pub enum KeyItem {
    None, // No key item
    Wall, // Jump over a wall
    FinishKey, // Must get before going to finish
    Diagonal, // Move diagonally
    BounceLess, // Bounce -1 less
    BounceMore, // Bounce +1 more
    BounceChange, // Change bounce direction
    TeleportKey(char), // Teleport to a portal
    DoorKey(char), // Open a door
    CloudKey(char), // Jump on air
}
