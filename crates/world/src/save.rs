use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::chunk::{BLOCKS_PER_CHUNK, Chunk};
use crate::position::ChunkPosition;
use crate::world::World;

pub const SAVE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    pub position: Vec3,
    pub yaw: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SaveError {
    UnsupportedVersion(u32),
    Corrupt,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::UnsupportedVersion(version) => {
                write!(formatter, "save version {version} is not supported")
            }
            SaveError::Corrupt => write!(formatter, "save data is corrupt"),
        }
    }
}

impl std::error::Error for SaveError {}

#[derive(Serialize, Deserialize)]
struct WorldSave {
    version: u32,
    player: PlayerState,
    chunks: Vec<ChunkSave>,
}

#[derive(Serialize, Deserialize)]
struct ChunkSave {
    position: ChunkPosition,
    runs: Vec<BlockRun>,
}

#[derive(Serialize, Deserialize)]
struct BlockRun {
    length: u16,
    block: Block,
}

pub fn encode(world: &World, player: PlayerState) -> Vec<u8> {
    let mut chunk_positions: Vec<_> = world.chunk_positions().collect();
    chunk_positions.sort_by_key(|chunk_position| chunk_position.0.to_array());
    let chunks = chunk_positions
        .into_iter()
        .filter_map(|position| {
            world.chunk(position).map(|chunk| ChunkSave {
                position,
                runs: run_length_encode(chunk),
            })
        })
        .collect();
    let save = WorldSave {
        version: SAVE_VERSION,
        player,
        chunks,
    };
    postcard::to_allocvec(&save).expect("encoding into a growable buffer cannot fail")
}

pub fn decode(bytes: &[u8]) -> Result<(World, PlayerState), SaveError> {
    let (version, _) = postcard::take_from_bytes::<u32>(bytes).map_err(|_| SaveError::Corrupt)?;
    if version != SAVE_VERSION {
        return Err(SaveError::UnsupportedVersion(version));
    }
    let save: WorldSave = postcard::from_bytes(bytes).map_err(|_| SaveError::Corrupt)?;
    let mut world = World::default();
    for chunk_save in save.chunks {
        let chunk = run_length_decode(&chunk_save.runs)?;
        world.insert_chunk(chunk_save.position, chunk);
    }
    Ok((world, save.player))
}

fn run_length_encode(chunk: &Chunk) -> Vec<BlockRun> {
    let mut runs: Vec<BlockRun> = Vec::new();
    for (_, block) in chunk.blocks() {
        match runs.last_mut() {
            Some(run) if run.block == block => run.length += 1,
            _ => runs.push(BlockRun { length: 1, block }),
        }
    }
    runs
}

fn run_length_decode(runs: &[BlockRun]) -> Result<Chunk, SaveError> {
    let mut chunk = Chunk::default();
    let mut filled = 0;
    for run in runs {
        let end = filled + usize::from(run.length);
        if end > BLOCKS_PER_CHUNK {
            return Err(SaveError::Corrupt);
        }
        chunk.fill(filled..end, run.block);
        filled = end;
    }
    if filled < BLOCKS_PER_CHUNK {
        return Err(SaveError::Corrupt);
    }
    Ok(chunk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::generate_flat_hills;
    use crate::position::BlockPosition;

    fn player() -> PlayerState {
        PlayerState {
            position: Vec3::new(1.5, 9.0, -2.5),
            yaw: 0.75,
        }
    }

    #[test]
    fn encoding_and_decoding_round_trips_the_world_and_the_player() {
        let mut world = generate_flat_hills(1, 42);
        world.set_block(BlockPosition::new(-3, 12, 7), Block::Brick);
        let bytes = encode(&world, player());
        let (decoded_world, decoded_player) = decode(&bytes).expect("valid save decodes");
        assert_eq!(decoded_world, world);
        assert_eq!(decoded_player, player());
    }

    #[test]
    fn run_length_encoding_keeps_a_chunk_far_smaller_than_its_block_count() {
        let world = generate_flat_hills(0, 42);
        let bytes = encode(&world, player());
        assert!(bytes.len() < 2 * 16 * 16 * 16 / 4, "{} bytes", bytes.len());
    }

    #[test]
    fn a_save_from_another_version_is_reported_as_unsupported() {
        let world = generate_flat_hills(0, 1);
        let mut bytes = encode(&world, player());
        bytes[0] = (SAVE_VERSION + 1) as u8;
        assert_eq!(
            decode(&bytes),
            Err(SaveError::UnsupportedVersion(SAVE_VERSION + 1))
        );
    }

    #[test]
    fn truncated_bytes_are_reported_as_corrupt() {
        let world = generate_flat_hills(0, 1);
        let bytes = encode(&world, player());
        assert_eq!(decode(&bytes[..bytes.len() / 2]), Err(SaveError::Corrupt));
        assert_eq!(decode(&[]), Err(SaveError::Corrupt));
    }

    #[test]
    fn a_chunk_whose_runs_do_not_fill_it_is_corrupt() {
        let runs = [BlockRun {
            length: 5,
            block: Block::Stone,
        }];
        assert_eq!(run_length_decode(&runs).err(), Some(SaveError::Corrupt));
    }

    #[test]
    fn a_chunk_whose_runs_overflow_it_is_corrupt() {
        let runs = [
            BlockRun {
                length: BLOCKS_PER_CHUNK as u16,
                block: Block::Stone,
            },
            BlockRun {
                length: 1,
                block: Block::Dirt,
            },
        ];
        assert_eq!(run_length_decode(&runs).err(), Some(SaveError::Corrupt));
    }

    #[test]
    fn the_save_format_is_pinned_byte_for_byte() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(0, 0, 0), Block::Stone);
        world.set_block(BlockPosition::new(1, 0, 0), Block::Stone);
        world.set_block(BlockPosition::new(2, 1, 0), Block::Grass);
        let player = PlayerState {
            position: Vec3::new(0.5, 2.0, -1.5),
            yaw: 0.5,
        };
        assert_eq!(
            encode(&world, player),
            [
                1, 0, 0, 0, 63, 0, 0, 0, 64, 0, 0, 192, 191, 0, 0, 0, 63, 1, 0, 0, 0, 4, 2, 3, 16,
                0, 1, 1, 237, 31, 0
            ]
        );
    }

    #[test]
    fn the_error_messages_read_as_sentences() {
        assert_eq!(
            SaveError::UnsupportedVersion(3).to_string(),
            "save version 3 is not supported"
        );
        assert_eq!(SaveError::Corrupt.to_string(), "save data is corrupt");
    }
}
