use std::collections::HashMap;

use crate::block::Block;
use crate::chunk::Chunk;
use crate::position::{BlockPosition, ChunkPosition};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    chunks: HashMap<ChunkPosition, Chunk>,
}

impl World {
    pub fn block_at(&self, block_position: BlockPosition) -> Block {
        let (chunk_position, local_position) = block_position.split();
        self.chunks
            .get(&chunk_position)
            .map_or(Block::Air, |chunk| chunk.get(local_position))
    }

    pub fn set_block(&mut self, block_position: BlockPosition, block: Block) {
        let (chunk_position, local_position) = block_position.split();
        self.chunks
            .entry(chunk_position)
            .or_default()
            .set(local_position, block);
    }

    pub fn insert_chunk(&mut self, chunk_position: ChunkPosition, chunk: Chunk) {
        self.chunks.insert(chunk_position, chunk);
    }

    pub fn chunk(&self, chunk_position: ChunkPosition) -> Option<&Chunk> {
        self.chunks.get(&chunk_position)
    }

    pub fn chunk_positions(&self) -> impl Iterator<Item = ChunkPosition> + '_ {
        self.chunks.keys().copied()
    }

    pub fn is_solid_at(&self, block_position: BlockPosition) -> bool {
        self.block_at(block_position).is_solid()
    }

    pub fn highest_solid_y(&self, x: i32, z: i32, search_from_y: i32) -> Option<i32> {
        let lowest_loaded_y = self
            .chunks
            .keys()
            .map(|chunk_position| chunk_position.min_block().0.y)
            .min()?;
        (lowest_loaded_y..=search_from_y)
            .rev()
            .find(|y| self.is_solid_at(BlockPosition::new(x, *y, z)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unloaded_chunk_reads_as_air() {
        let world = World::default();
        assert_eq!(world.block_at(BlockPosition::new(1000, -5, 3)), Block::Air);
        assert_eq!(world.chunk_positions().count(), 0);
    }

    #[test]
    fn setting_a_block_creates_its_chunk_and_reads_back() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(-1, 0, 0), Block::Stone);
        assert_eq!(world.block_at(BlockPosition::new(-1, 0, 0)), Block::Stone);
        assert_eq!(
            world.chunk_positions().collect::<Vec<_>>(),
            vec![ChunkPosition::new(-1, 0, 0)]
        );
    }

    #[test]
    fn blocks_on_both_sides_of_a_chunk_border_live_in_different_chunks() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(15, 0, 0), Block::Grass);
        world.set_block(BlockPosition::new(16, 0, 0), Block::Dirt);
        assert_eq!(world.block_at(BlockPosition::new(15, 0, 0)), Block::Grass);
        assert_eq!(world.block_at(BlockPosition::new(16, 0, 0)), Block::Dirt);
        assert_eq!(world.chunk_positions().count(), 2);
    }

    #[test]
    fn the_highest_solid_block_in_a_column_is_found_below_the_search_start() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(4, 2, 4), Block::Stone);
        world.set_block(BlockPosition::new(4, 9, 4), Block::Grass);
        world.set_block(BlockPosition::new(4, 30, 4), Block::Leaves);
        assert_eq!(world.highest_solid_y(4, 4, 20), Some(9));
        assert_eq!(world.highest_solid_y(4, 4, 40), Some(30));
        assert_eq!(world.highest_solid_y(5, 4, 40), None);
        assert_eq!(World::default().highest_solid_y(0, 0, 40), None);
    }

    #[test]
    fn overwriting_with_air_leaves_the_position_empty() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(2, 2, 2), Block::Brick);
        world.set_block(BlockPosition::new(2, 2, 2), Block::Air);
        assert!(!world.is_solid_at(BlockPosition::new(2, 2, 2)));
    }
}
