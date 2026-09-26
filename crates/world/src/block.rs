use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
    Leaves,
    Brick,
}

impl Block {
    pub const fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }

    pub const fn color(self) -> [f32; 4] {
        match self {
            Block::Air => [0.0, 0.0, 0.0, 0.0],
            Block::Grass => [0.55, 0.82, 0.42, 1.0],
            Block::Dirt => [0.62, 0.46, 0.33, 1.0],
            Block::Stone => [0.66, 0.68, 0.72, 1.0],
            Block::Sand => [0.94, 0.87, 0.62, 1.0],
            Block::Wood => [0.52, 0.36, 0.24, 1.0],
            Block::Leaves => [0.36, 0.70, 0.40, 1.0],
            Block::Brick => [0.87, 0.47, 0.42, 1.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_is_the_only_block_that_is_not_solid() {
        assert!(!Block::Air.is_solid());
        for block in [
            Block::Grass,
            Block::Dirt,
            Block::Stone,
            Block::Sand,
            Block::Wood,
            Block::Leaves,
            Block::Brick,
        ] {
            assert!(block.is_solid(), "{block:?} should be solid");
        }
    }

    #[test]
    fn every_solid_block_has_an_opaque_color() {
        for block in [
            Block::Grass,
            Block::Dirt,
            Block::Stone,
            Block::Sand,
            Block::Wood,
            Block::Leaves,
            Block::Brick,
        ] {
            assert_eq!(block.color()[3], 1.0, "{block:?} should be opaque");
        }
    }
}
