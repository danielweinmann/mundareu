use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

use crate::chunk::CHUNK_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPosition(pub IVec3);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPosition(pub IVec3);

impl BlockPosition {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn containing(point: Vec3) -> Self {
        Self(point.floor().as_ivec3())
    }

    pub fn chunk_position(self) -> ChunkPosition {
        ChunkPosition(self.0.div_euclid(IVec3::splat(CHUNK_SIZE)))
    }

    pub fn local_position(self) -> IVec3 {
        self.0.rem_euclid(IVec3::splat(CHUNK_SIZE))
    }

    pub fn split(self) -> (ChunkPosition, IVec3) {
        (self.chunk_position(), self.local_position())
    }

    pub fn offset(self, direction: IVec3) -> Self {
        Self(self.0 + direction)
    }

    pub fn min_corner(self) -> Vec3 {
        self.0.as_vec3()
    }
}

impl ChunkPosition {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn block_position(self, local_position: IVec3) -> BlockPosition {
        BlockPosition(self.0 * CHUNK_SIZE + local_position)
    }

    pub fn min_block(self) -> BlockPosition {
        self.block_position(IVec3::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_block_positions_map_to_their_chunk_and_local_offset() {
        let (chunk_position, local_position) = BlockPosition::new(17, 3, 40).split();
        assert_eq!(chunk_position, ChunkPosition::new(1, 0, 2));
        assert_eq!(local_position, IVec3::new(1, 3, 8));
    }

    #[test]
    fn negative_block_positions_floor_into_the_previous_chunk() {
        let (chunk_position, local_position) = BlockPosition::new(-1, -16, -17).split();
        assert_eq!(chunk_position, ChunkPosition::new(-1, -1, -2));
        assert_eq!(local_position, IVec3::new(15, 0, 15));
    }

    #[test]
    fn chunk_and_local_position_recompose_the_block_position() {
        for block_position in [
            BlockPosition::new(0, 0, 0),
            BlockPosition::new(-1, 5, 33),
            BlockPosition::new(-100, -17, 15),
        ] {
            let (chunk_position, local_position) = block_position.split();
            assert_eq!(
                chunk_position.block_position(local_position),
                block_position
            );
        }
    }

    #[test]
    fn containing_floors_fractional_points_including_negative_ones() {
        assert_eq!(
            BlockPosition::containing(Vec3::new(1.9, -0.1, -2.5)),
            BlockPosition::new(1, -1, -3)
        );
    }
}
