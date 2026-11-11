//! Discrete integer grid coordinate model and fixed-point progress tracking.

use bevy_ecs::component::Component;

/// 3D integer tile coordinate representing a discrete spatial unit on Aethelgard.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl TileCoord {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the coordinate offset by one step in the given direction.
    #[inline]
    pub const fn step(self, dir: Direction) -> Self {
        match dir {
            Direction::North => Self::new(self.x, self.y + 1, self.z),
            Direction::South => Self::new(self.x, self.y - 1, self.z),
            Direction::East => Self::new(self.x + 1, self.y, self.z),
            Direction::West => Self::new(self.x - 1, self.y, self.z),
            Direction::Up => Self::new(self.x, self.y, self.z + 1),
            Direction::Down => Self::new(self.x, self.y, self.z - 1),
        }
    }

    /// Manhattan distance between two grid points.
    #[inline]
    pub const fn manhattan_distance(self, other: Self) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) as u32
    }
}

/// Cardinal and vertical spatial directions on the Aethelgard grid.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl Direction {
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

/// Fixed-point sub-tile progress tracking along a conveyor belt.
///
/// 1 full tile traversal corresponds to `65,536` units (`2^16`).
/// This completely eliminates IEEE-754 floating-point drift across platforms.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Component, Default)]
pub struct SubTileProgress(pub u32);

impl SubTileProgress {
    /// Progress value corresponding to having fully traversed one grid tile.
    pub const FULL_TILE: u32 = 65_536;

    #[inline]
    pub const fn new(units: u32) -> Self {
        Self(units)
    }

    #[inline]
    pub const fn is_complete(&self) -> bool {
        self.0 >= Self::FULL_TILE
    }

    #[inline]
    pub fn advance(&mut self, delta_units: u32) {
        self.0 = self.0.saturating_add(delta_units);
    }

    #[inline]
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}
