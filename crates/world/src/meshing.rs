use glam::IVec3;

use crate::position::ChunkPosition;
use crate::world::World;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn face_count(&self) -> usize {
        self.indices.len() / 6
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

struct Face {
    normal: IVec3,
    tangent: IVec3,
    bitangent: IVec3,
    shade: f32,
}

const FACES: [Face; 6] = [
    Face {
        normal: IVec3::Y,
        tangent: IVec3::Z,
        bitangent: IVec3::X,
        shade: 1.0,
    },
    Face {
        normal: IVec3::NEG_Y,
        tangent: IVec3::X,
        bitangent: IVec3::Z,
        shade: 0.55,
    },
    Face {
        normal: IVec3::X,
        tangent: IVec3::Y,
        bitangent: IVec3::Z,
        shade: 0.82,
    },
    Face {
        normal: IVec3::NEG_X,
        tangent: IVec3::Z,
        bitangent: IVec3::Y,
        shade: 0.82,
    },
    Face {
        normal: IVec3::Z,
        tangent: IVec3::X,
        bitangent: IVec3::Y,
        shade: 0.72,
    },
    Face {
        normal: IVec3::NEG_Z,
        tangent: IVec3::Y,
        bitangent: IVec3::X,
        shade: 0.72,
    },
];

pub fn mesh_chunk(world: &World, chunk_position: ChunkPosition) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let Some(chunk) = world.chunk(chunk_position) else {
        return mesh;
    };
    for (local_position, block) in chunk.blocks() {
        if !block.is_solid() {
            continue;
        }
        let block_position = chunk_position.block_position(local_position);
        for face in &FACES {
            if world.is_solid_at(block_position.offset(face.normal)) {
                continue;
            }
            emit_face(&mut mesh, block_position.0, face, block.color());
        }
    }
    mesh
}

fn emit_face(mesh: &mut ChunkMesh, block_position: IVec3, face: &Face, color: [f32; 4]) {
    let first_index = mesh.positions.len() as u32;
    let corner = block_position + face.normal.max(IVec3::ZERO);
    let shaded = [
        color[0] * face.shade,
        color[1] * face.shade,
        color[2] * face.shade,
        color[3],
    ];
    for offset in [
        IVec3::ZERO,
        face.tangent,
        face.tangent + face.bitangent,
        face.bitangent,
    ] {
        mesh.positions.push((corner + offset).as_vec3().to_array());
        mesh.normals.push(face.normal.as_vec3().to_array());
        mesh.colors.push(shaded);
    }
    mesh.indices.extend(
        [0, 1, 2, 0, 2, 3]
            .iter()
            .map(|vertex_offset| first_index + vertex_offset),
    );
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;
    use crate::block::Block;
    use crate::position::BlockPosition;

    #[test]
    fn an_isolated_block_emits_six_faces() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(3, 3, 3), Block::Stone);
        let mesh = mesh_chunk(&world, ChunkPosition::new(0, 0, 0));
        assert_eq!(mesh.face_count(), 6);
        assert_eq!(mesh.positions.len(), 24);
        assert_eq!(mesh.normals.len(), 24);
        assert_eq!(mesh.colors.len(), 24);
    }

    #[test]
    fn two_adjacent_blocks_hide_their_shared_faces() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(3, 3, 3), Block::Stone);
        world.set_block(BlockPosition::new(4, 3, 3), Block::Stone);
        assert_eq!(
            mesh_chunk(&world, ChunkPosition::new(0, 0, 0)).face_count(),
            10
        );
    }

    #[test]
    fn a_face_touching_a_solid_block_in_the_neighbor_chunk_is_culled() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(15, 3, 3), Block::Stone);
        world.set_block(BlockPosition::new(16, 3, 3), Block::Stone);
        assert_eq!(
            mesh_chunk(&world, ChunkPosition::new(0, 0, 0)).face_count(),
            5
        );
        assert_eq!(
            mesh_chunk(&world, ChunkPosition::new(1, 0, 0)).face_count(),
            5
        );
    }

    #[test]
    fn an_empty_or_missing_chunk_yields_an_empty_mesh() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(0, 0, 0), Block::Air);
        assert!(mesh_chunk(&world, ChunkPosition::new(0, 0, 0)).is_empty());
        assert!(mesh_chunk(&world, ChunkPosition::new(9, 9, 9)).is_empty());
    }

    #[test]
    fn positions_are_in_world_space_and_faces_wind_counterclockwise_outward() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(-16, 0, 0), Block::Grass);
        let mesh = mesh_chunk(&world, ChunkPosition::new(-1, 0, 0));
        for triangle in mesh.indices.chunks(3) {
            let [a, b, c] =
                [triangle[0], triangle[1], triangle[2]].map(|index| mesh.positions[index as usize]);
            let normal = Vec3::from(mesh.normals[triangle[0] as usize]);
            let winding = (Vec3::from(b) - Vec3::from(a)).cross(Vec3::from(c) - Vec3::from(a));
            assert!(
                winding.dot(normal) > 0.0,
                "triangle winds against its normal"
            );
            for vertex in [a, b, c] {
                assert!((-16.0..=-15.0).contains(&vertex[0]), "x {vertex:?}");
            }
        }
    }

    #[test]
    fn the_top_face_is_brighter_than_the_bottom_face() {
        let mut world = World::default();
        world.set_block(BlockPosition::new(0, 0, 0), Block::Grass);
        let mesh = mesh_chunk(&world, ChunkPosition::new(0, 0, 0));
        let brightness = |normal: [f32; 3]| {
            let index = mesh
                .normals
                .iter()
                .position(|candidate| *candidate == normal)
                .expect("face present");
            mesh.colors[index][0] + mesh.colors[index][1] + mesh.colors[index][2]
        };
        assert!(brightness([0.0, 1.0, 0.0]) > brightness([1.0, 0.0, 0.0]));
        assert!(brightness([1.0, 0.0, 0.0]) > brightness([0.0, -1.0, 0.0]));
    }
}
