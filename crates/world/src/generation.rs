use glam::{FloatExt, IVec3};

use crate::block::Block;
use crate::chunk::{CHUNK_SIZE, Chunk};
use crate::position::{BlockPosition, ChunkPosition};
use crate::world::World;

const BASE_HEIGHT: f32 = 8.0;
const HILL_AMPLITUDE: f32 = 5.0;
const HILL_WAVELENGTH: f32 = 28.0;
const DETAIL_AMPLITUDE: f32 = 1.5;
const DETAIL_WAVELENGTH: f32 = 9.0;
const SAND_BELOW_HEIGHT: i32 = 6;
const DIRT_DEPTH: i32 = 3;
const CHUNKS_TALL: i32 = 2;
const TREE_CHANCE_PER_COLUMN: u64 = 180;
const TREE_TRUNK_HEIGHT: i32 = 4;
const TREE_MARGIN_FROM_EDGE: i32 = 2;

pub fn generate_flat_hills(radius_in_chunks: i32, seed: u64) -> World {
    let mut world = World::default();
    for chunk_x in -radius_in_chunks..=radius_in_chunks {
        for chunk_z in -radius_in_chunks..=radius_in_chunks {
            for chunk_y in 0..CHUNKS_TALL {
                let chunk_position = ChunkPosition::new(chunk_x, chunk_y, chunk_z);
                world.insert_chunk(chunk_position, generate_chunk(chunk_position, seed));
            }
        }
    }
    plant_trees(&mut world, radius_in_chunks, seed);
    world
}

pub fn surface_height(x: i32, z: i32, seed: u64) -> i32 {
    let hills = value_noise(x as f32 / HILL_WAVELENGTH, z as f32 / HILL_WAVELENGTH, seed);
    let detail = value_noise(
        x as f32 / DETAIL_WAVELENGTH,
        z as f32 / DETAIL_WAVELENGTH,
        seed.wrapping_add(1),
    );
    (BASE_HEIGHT + hills * HILL_AMPLITUDE + detail * DETAIL_AMPLITUDE).round() as i32
}

fn generate_chunk(chunk_position: ChunkPosition, seed: u64) -> Chunk {
    let mut chunk = Chunk::default();
    let min_block = chunk_position.min_block().0;
    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            let height = surface_height(min_block.x + local_x, min_block.z + local_z, seed);
            for local_y in 0..CHUNK_SIZE {
                let block = block_for_column(min_block.y + local_y, height);
                if block.is_solid() {
                    chunk.set(IVec3::new(local_x, local_y, local_z), block);
                }
            }
        }
    }
    chunk
}

fn block_for_column(y: i32, height: i32) -> Block {
    if y > height {
        Block::Air
    } else if y == height {
        if height <= SAND_BELOW_HEIGHT {
            Block::Sand
        } else {
            Block::Grass
        }
    } else if y > height - DIRT_DEPTH {
        Block::Dirt
    } else {
        Block::Stone
    }
}

fn plant_trees(world: &mut World, radius_in_chunks: i32, seed: u64) {
    let extent = radius_in_chunks * CHUNK_SIZE + CHUNK_SIZE - TREE_MARGIN_FROM_EDGE;
    let start = -radius_in_chunks * CHUNK_SIZE + TREE_MARGIN_FROM_EDGE;
    for x in start..extent {
        for z in start..extent {
            let height = surface_height(x, z, seed);
            let grows_here =
                hash(x, z, seed.wrapping_add(2)).is_multiple_of(TREE_CHANCE_PER_COLUMN);
            if grows_here && height > SAND_BELOW_HEIGHT {
                plant_tree(world, BlockPosition::new(x, height + 1, z));
            }
        }
    }
}

fn plant_tree(world: &mut World, base: BlockPosition) {
    for trunk_y in 0..TREE_TRUNK_HEIGHT {
        world.set_block(base.offset(IVec3::new(0, trunk_y, 0)), Block::Wood);
    }
    let canopy_center = base.offset(IVec3::new(0, TREE_TRUNK_HEIGHT, 0));
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let leaf_position = canopy_center.offset(IVec3::new(x, y, z));
                if !world.is_solid_at(leaf_position) {
                    world.set_block(leaf_position, Block::Leaves);
                }
            }
        }
    }
}

fn value_noise(x: f32, z: f32, seed: u64) -> f32 {
    let cell_x = x.floor();
    let cell_z = z.floor();
    let fraction_x = smoothstep(x - cell_x);
    let fraction_z = smoothstep(z - cell_z);
    let corner = |offset_x: i32, offset_z: i32| {
        lattice_value(cell_x as i32 + offset_x, cell_z as i32 + offset_z, seed)
    };
    let near = corner(0, 0).lerp(corner(1, 0), fraction_x);
    let far = corner(0, 1).lerp(corner(1, 1), fraction_x);
    near.lerp(far, fraction_z) * 2.0 - 1.0
}

fn lattice_value(x: i32, z: i32, seed: u64) -> f32 {
    (hash(x, z, seed) % 10_000) as f32 / 10_000.0
}

fn hash(x: i32, z: i32, seed: u64) -> u64 {
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    for value in [x as u32 as u64, z as u32 as u64] {
        state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
        state = state.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        state ^= state >> 31;
    }
    state
}

fn smoothstep(fraction: f32) -> f32 {
    fraction * fraction * (3.0 - 2.0 * fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_produces_the_same_world() {
        assert_eq!(generate_flat_hills(1, 7), generate_flat_hills(1, 7));
    }

    #[test]
    fn different_seeds_produce_different_worlds() {
        assert_ne!(generate_flat_hills(1, 7), generate_flat_hills(1, 8));
    }

    #[test]
    fn the_world_covers_the_requested_radius_in_chunks() {
        let world = generate_flat_hills(2, 1);
        assert_eq!(
            world.chunk_positions().count(),
            (5 * 5 * CHUNKS_TALL) as usize
        );
        assert!(world.chunk(ChunkPosition::new(-2, 0, 2)).is_some());
        assert!(world.chunk(ChunkPosition::new(3, 0, 0)).is_none());
    }

    #[test]
    fn every_column_has_a_surface_block_over_dirt_over_stone() {
        let world = generate_flat_hills(1, 3);
        for x in [-16, -1, 0, 9, 31] {
            for z in [-16, 0, 15, 31] {
                let height = surface_height(x, z, 3);
                let surface = world.block_at(BlockPosition::new(x, height, z));
                assert!(matches!(surface, Block::Grass | Block::Sand), "{surface:?}");
                assert_eq!(
                    world.block_at(BlockPosition::new(x, height + 1, z)),
                    Block::Air
                );
                assert_eq!(
                    world.block_at(BlockPosition::new(x, height - 1, z)),
                    Block::Dirt
                );
                assert_eq!(world.block_at(BlockPosition::new(x, 0, z)), Block::Stone);
            }
        }
    }

    #[test]
    fn hills_stay_within_the_gentle_range() {
        for x in -64..64 {
            for z in -64..64 {
                let height = surface_height(x, z, 11);
                assert!((1..=15).contains(&height), "height {height} at {x},{z}");
            }
        }
    }

    #[test]
    fn a_handful_of_trees_grow_with_wood_trunks_under_leaves() {
        let world = generate_flat_hills(3, 5);
        let mut trunks = 0;
        for chunk_position in world.chunk_positions() {
            let chunk = world.chunk(chunk_position).expect("listed chunk exists");
            for (local_position, block) in chunk.blocks() {
                if block == Block::Wood {
                    let below = chunk_position.block_position(local_position - IVec3::Y);
                    let below_block = world.block_at(below);
                    assert!(
                        matches!(below_block, Block::Wood | Block::Grass),
                        "{below_block:?}"
                    );
                    if below_block == Block::Grass {
                        let mut above_trunk = chunk_position.block_position(local_position);
                        while world.block_at(above_trunk) == Block::Wood {
                            above_trunk = above_trunk.offset(IVec3::Y);
                        }
                        assert_eq!(world.block_at(above_trunk), Block::Leaves);
                        trunks += 1;
                    }
                }
            }
        }
        assert!(trunks >= 3, "expected a handful of trees, found {trunks}");
    }
}
