use crate::constants::SECTOR_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2 {
    pub x: u16,
    pub y: u16,
}

impl Vec2 {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn from_tile_id(tile_id: u64) -> Self {
        Self {
            x: ((tile_id >> 16) & 0xFFFF) as u16,
            y: (tile_id & 0xFFFF) as u16,
        }
    }

    pub fn tile_id(&self) -> u64 {
        ((self.x as u64) << 16) | (self.y as u64)
    }

    pub fn sector_key(&self) -> u64 {
        let sector_x = self.x / SECTOR_SIZE;
        let sector_y = self.y / SECTOR_SIZE;
        ((sector_x as u64) << 16) | (sector_y as u64)
    }
}

impl From<(u16, u16)> for Vec2 {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_id_roundtrip() {
        let pos = Vec2::new(1152, 1152);
        assert_eq!(Vec2::from_tile_id(pos.tile_id()), pos);
    }

    #[test]
    fn tile_id_roundtrip_extremes() {
        let pos = Vec2::new(u16::MAX, u16::MAX);
        assert_eq!(Vec2::from_tile_id(pos.tile_id()), pos);

        let pos = Vec2::new(0, 0);
        assert_eq!(Vec2::from_tile_id(pos.tile_id()), pos);
    }

    #[test]
    fn sector_key_groups_nearby_points() {
        let a = Vec2::new(100, 100).sector_key();
        let b = Vec2::new(200, 200).sector_key();
        assert_eq!(a, b);
    }

    #[test]
    fn sector_key_separates_distant_points() {
        let a = Vec2::new(0, 0).sector_key();
        let b = Vec2::new(256, 256).sector_key();
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
}
