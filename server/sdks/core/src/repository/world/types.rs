use crate::{
    constants::SECTOR_SIZE,
    repository::world::{MapV1, WalkedMapChunkV1},
};
use spacetimedb::SpacetimeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
    pub x: u16,
    pub y: u16,
}

impl Vec2 {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn with_z(self, z: u8) -> Vec3 {
        Vec3 { x: self.x, y: self.y, z }
    }
}

impl From<(u16, u16)> for Vec2 {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
    }
}

impl From<Vec3> for Vec2 {
    fn from(vec3: Vec3) -> Self {
        Self { x: vec3.x, y: vec3.y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3 {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl Vec3 {
    pub fn new(x: u16, y: u16, z: u8) -> Self {
        Self { x, y, z }
    }

    pub fn from_map_id(map_id: u64) -> Self {
        Self {
            z: (map_id >> 32) as u8,
            x: ((map_id >> 16) & 0xFFFF) as u16,
            y: (map_id & 0xFFFF) as u16,
        }
    }

    pub fn map_id(&self) -> u64 {
        ((self.z as u64) << 32) | ((self.x as u64) << 16) | (self.y as u64)
    }

    pub fn sector_key(&self) -> u64 {
        let sector_x = self.x / SECTOR_SIZE;
        let sector_y = self.y / SECTOR_SIZE;
        ((self.z as u64) << 32) | ((sector_x as u64) << 16) | (sector_y as u64)
    }
}

impl From<(u16, u16, u8)> for Vec3 {
    fn from((x, y, z): (u16, u16, u8)) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Self {
        Self {
            min: Vec2::new(x1, y1),
            max: Vec2::new(x2, y2),
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x && point.y >= self.min.y && point.y <= self.max.y
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x && self.min.y <= other.max.y && self.max.y >= other.min.y
    }
}

impl From<&MapV1> for Rect {
    fn from(chunk: &MapV1) -> Self {
        Self::new(chunk.x1, chunk.y1, chunk.x2, chunk.y2)
    }
}

impl From<&WalkedMapChunkV1> for Rect {
    fn from(cache: &WalkedMapChunkV1) -> Self {
        Self::new(cache.x1, cache.y1, cache.x2, cache.y2)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SpacetimeType)]
pub enum DirectionV1 {
    North,
    East,
    #[default]
    South,
    West,
}

impl From<MovementV1> for DirectionV1 {
    fn from(movement: MovementV1) -> Self {
        match movement {
            MovementV1::North | MovementV1::NorthEast => DirectionV1::North,
            MovementV1::East | MovementV1::SouthEast => DirectionV1::East,
            MovementV1::South | MovementV1::SouthWest => DirectionV1::South,
            MovementV1::West | MovementV1::NorthWest => DirectionV1::West,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SpacetimeType)]
pub enum MovementV1 {
    North,
    NorthEast,
    East,
    SouthEast,
    #[default]
    South,
    SouthWest,
    West,
    NorthWest,
}

impl MovementV1 {
    pub fn is_diagonal(&self) -> bool {
        matches!(self, Self::NorthEast | Self::SouthEast | Self::SouthWest | Self::NorthWest)
    }

    pub fn translate(&self, x: u16, y: u16) -> (u16, u16) {
        match self {
            MovementV1::North => (x, y.saturating_sub(1)),
            MovementV1::NorthEast => (x.saturating_add(1), y.saturating_sub(1)),
            MovementV1::East => (x.saturating_add(1), y),
            MovementV1::SouthEast => (x.saturating_add(1), y.saturating_add(1)),
            MovementV1::South => (x, y.saturating_add(1)),
            MovementV1::SouthWest => (x.saturating_sub(1), y.saturating_add(1)),
            MovementV1::West => (x.saturating_sub(1), y),
            MovementV1::NorthWest => (x.saturating_sub(1), y.saturating_sub(1)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SpacetimeType)]
pub enum MapTileV1 {
    Water,
    Grass,
}

impl MapTileV1 {
    pub fn is_walkable(&self) -> bool {
        matches!(self, MapTileV1::Grass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_id_roundtrip() {
        let pos = Vec3::new(1152, 1152, 127);
        assert_eq!(Vec3::from_map_id(pos.map_id()), pos);
    }

    #[test]
    fn map_id_roundtrip_extremes() {
        let pos = Vec3::new(u16::MAX, u16::MAX, u8::MAX);
        assert_eq!(Vec3::from_map_id(pos.map_id()), pos);

        let pos = Vec3::new(0, 0, 0);
        assert_eq!(Vec3::from_map_id(pos.map_id()), pos);
    }

    #[test]
    fn sector_key_groups_nearby_points() {
        let a = Vec3::new(100, 100, 127).sector_key();
        let b = Vec3::new(200, 200, 127).sector_key();
        assert_eq!(a, b);
    }

    #[test]
    fn sector_key_separates_distant_points() {
        let a = Vec3::new(0, 0, 127).sector_key();
        let b = Vec3::new(256, 256, 127).sector_key();
        assert_ne!(a, b);
    }

    #[test]
    fn sector_key_separates_z_levels() {
        let a = Vec3::new(100, 100, 0).sector_key();
        let b = Vec3::new(100, 100, 1).sector_key();
        assert_ne!(a, b);
    }

    #[test]
    fn rect_contains_inside() {
        assert!(Rect::new(0, 0, 100, 100).contains(Vec2::new(50, 50)));
    }

    #[test]
    fn rect_contains_on_edges() {
        let rect = Rect::new(0, 0, 100, 100);
        assert!(rect.contains(Vec2::new(0, 0)));
        assert!(rect.contains(Vec2::new(100, 100)));
    }

    #[test]
    fn rect_contains_outside() {
        let rect = Rect::new(0, 0, 100, 100);
        assert!(!rect.contains(Vec2::new(101, 50)));
        assert!(!rect.contains(Vec2::new(50, 101)));
    }

    #[test]
    fn rect_overlaps_overlapping() {
        assert!(Rect::new(0, 0, 50, 50).overlaps(&Rect::new(25, 25, 75, 75)));
    }

    #[test]
    fn rect_overlaps_touching_edge() {
        assert!(Rect::new(0, 0, 50, 50).overlaps(&Rect::new(50, 50, 100, 100)));
    }

    #[test]
    fn rect_overlaps_no_overlap() {
        assert!(!Rect::new(0, 0, 50, 50).overlaps(&Rect::new(51, 51, 100, 100)));
    }

    #[test]
    fn rect_overlaps_contained() {
        assert!(Rect::new(0, 0, 100, 100).overlaps(&Rect::new(25, 25, 75, 75)));
    }
}
