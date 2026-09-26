use std::ops::Range;

use glam::IVec3;

use crate::block::Block;

pub const CHUNK_SIZE: i32 = 16;
pub(crate) const BLOCKS_PER_CHUNK: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    blocks: Box<[Block; BLOCKS_PER_CHUNK]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: Box::new([Block::Air; BLOCKS_PER_CHUNK]),
        }
    }
}

impl Chunk {
    pub fn get(&self, local_position: IVec3) -> Block {
        self.blocks[index_of(local_position)]
    }

    pub fn set(&mut self, local_position: IVec3, block: Block) {
        self.blocks[index_of(local_position)] = block;
    }

    pub fn blocks(&self) -> impl Iterator<Item = (IVec3, Block)> + '_ {
        self.blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (local_position_of(index), *block))
    }

    pub(crate) fn fill(&mut self, indices: Range<usize>, block: Block) {
        self.blocks[indices].fill(block);
    }
}

fn index_of(local_position: IVec3) -> usize {
    let bounds = 0..CHUNK_SIZE;
    assert!(
        bounds.contains(&local_position.x)
            && bounds.contains(&local_position.y)
            && bounds.contains(&local_position.z),
        "local position {local_position} lies outside the chunk"
    );
    (local_position.x + local_position.y * CHUNK_SIZE + local_position.z * CHUNK_SIZE * CHUNK_SIZE)
        as usize
}

fn local_position_of(index: usize) -> IVec3 {
    let index = index as i32;
    IVec3::new(
        index % CHUNK_SIZE,
        (index / CHUNK_SIZE) % CHUNK_SIZE,
        index / (CHUNK_SIZE * CHUNK_SIZE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_chunk_is_all_air() {
        let chunk = Chunk::default();
        assert!(chunk.blocks().all(|(_, block)| block == Block::Air));
    }

    #[test]
    fn setting_a_block_reads_back_at_the_same_local_position() {
        let mut chunk = Chunk::default();
        chunk.set(IVec3::new(3, 7, 11), Block::Stone);
        assert_eq!(chunk.get(IVec3::new(3, 7, 11)), Block::Stone);
        assert_eq!(chunk.get(IVec3::new(11, 7, 3)), Block::Air);
    }

    #[test]
    fn every_local_position_has_its_own_slot() {
        let mut chunk = Chunk::default();
        chunk.set(IVec3::new(0, 0, 0), Block::Grass);
        chunk.set(IVec3::new(15, 0, 0), Block::Dirt);
        chunk.set(IVec3::new(0, 15, 0), Block::Stone);
        chunk.set(IVec3::new(0, 0, 15), Block::Sand);
        assert_eq!(chunk.get(IVec3::new(0, 0, 0)), Block::Grass);
        assert_eq!(chunk.get(IVec3::new(15, 0, 0)), Block::Dirt);
        assert_eq!(chunk.get(IVec3::new(0, 15, 0)), Block::Stone);
        assert_eq!(chunk.get(IVec3::new(0, 0, 15)), Block::Sand);
    }

    #[test]
    fn iterating_blocks_yields_each_position_with_its_block() {
        let mut chunk = Chunk::default();
        chunk.set(IVec3::new(1, 2, 3), Block::Brick);
        let bricks: Vec<_> = chunk
            .blocks()
            .filter(|(_, block)| *block == Block::Brick)
            .collect();
        assert_eq!(bricks, vec![(IVec3::new(1, 2, 3), Block::Brick)]);
        assert_eq!(chunk.blocks().count(), BLOCKS_PER_CHUNK);
    }

    #[test]
    #[should_panic(expected = "lies outside the chunk")]
    fn reading_outside_the_chunk_is_a_programming_error() {
        Chunk::default().get(IVec3::new(16, 0, 0));
    }
}
