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
    fn overwriting_with_air_leaves_the_position_empty() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(2, 2, 2), Block::Brick);
        world.set_block(BlockPosition::new(2, 2, 2), Block::Air);
        assert!(!world.is_solid_at(BlockPosition::new(2, 2, 2)));
    }
}
