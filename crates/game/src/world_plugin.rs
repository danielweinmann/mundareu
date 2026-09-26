use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use mundareu_world::{BlockPosition, CHUNK_SIZE, ChunkMesh, ChunkPosition, mesh_chunk};

use crate::GameSystems;
use crate::avatar_plugin::Avatar;

pub const RENDER_DISTANCE_IN_CHUNKS: i32 = 5;
const SKY_COLOR: Color = Color::srgb(0.58, 0.80, 0.98);
const SUNLIGHT_ILLUMINANCE_IN_LUX: f32 = 3_200.0;
const AMBIENT_BRIGHTNESS: f32 = 700.0;

#[derive(Resource)]
pub struct Terrain(pub mundareu_world::World);

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockChanged {
    pub block_position: BlockPosition,
}

#[derive(Resource, Default)]
struct ChunkViews {
    entities: HashMap<ChunkPosition, Option<Entity>>,
    center: Option<ChunkPosition>,
}

#[derive(Resource)]
struct TerrainMaterial(Handle<StandardMaterial>);

pub fn plugin(app: &mut App) {
    app.add_message::<BlockChanged>()
        .init_resource::<ChunkViews>()
        .insert_resource(ClearColor(SKY_COLOR))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: AMBIENT_BRIGHTNESS,
            ..default()
        })
        .add_systems(Startup, (setup_sunlight, setup_terrain_material))
        .add_systems(
            Update,
            (remesh_changed_chunks, stream_chunks_around_avatar)
                .chain()
                .in_set(GameSystems::Terrain),
        );
}

fn setup_sunlight(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: SUNLIGHT_ILLUMINANCE_IN_LUX,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-0.4, -1.0, -0.6), Vec3::Y),
    ));
}

fn setup_terrain_material(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.95,
        reflectance: 0.05,
        ..default()
    });
    commands.insert_resource(TerrainMaterial(material));
}

fn remesh_changed_chunks(
    mut commands: Commands,
    mut block_changes: MessageReader<BlockChanged>,
    terrain: Res<Terrain>,
    mut chunk_views: ResMut<ChunkViews>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TerrainMaterial>,
) {
    let dirty: HashSet<ChunkPosition> = block_changes
        .read()
        .flat_map(|change| chunks_touched_by(change.block_position))
        .collect();
    let Some(center) = chunk_views.center else {
        return;
    };
    for chunk_position in dirty {
        if let Some(Some(entity)) = chunk_views.entities.remove(&chunk_position) {
            commands.entity(entity).despawn();
        }
        if within_render_distance(chunk_position, center) {
            let entity = spawn_chunk_view(
                &mut commands,
                &terrain,
                chunk_position,
                &mut meshes,
                &material,
            );
            chunk_views.entities.insert(chunk_position, entity);
        }
    }
}

fn stream_chunks_around_avatar(
    mut commands: Commands,
    terrain: Res<Terrain>,
    mut chunk_views: ResMut<ChunkViews>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TerrainMaterial>,
    avatar: Single<&Avatar>,
) {
    let center = BlockPosition::containing(avatar.body.position).chunk_position();
    if chunk_views.center == Some(center) {
        return;
    }
    chunk_views.center = Some(center);
    let desired: HashSet<ChunkPosition> = terrain
        .0
        .chunk_positions()
        .filter(|chunk_position| within_render_distance(*chunk_position, center))
        .collect();
    chunk_views.entities.retain(|chunk_position, entity| {
        let keep = desired.contains(chunk_position);
        if !keep && let Some(entity) = entity {
            commands.entity(*entity).despawn();
        }
        keep
    });
    for chunk_position in desired {
        if chunk_views.entities.contains_key(&chunk_position) {
            continue;
        }
        let entity = spawn_chunk_view(
            &mut commands,
            &terrain,
            chunk_position,
            &mut meshes,
            &material,
        );
        chunk_views.entities.insert(chunk_position, entity);
    }
}

fn spawn_chunk_view(
    commands: &mut Commands,
    terrain: &Terrain,
    chunk_position: ChunkPosition,
    meshes: &mut Assets<Mesh>,
    material: &TerrainMaterial,
) -> Option<Entity> {
    let chunk_mesh = mesh_chunk(&terrain.0, chunk_position);
    if chunk_mesh.is_empty() {
        return None;
    }
    let entity = commands
        .spawn((
            Mesh3d(meshes.add(render_mesh(chunk_mesh))),
            MeshMaterial3d(material.0.clone()),
        ))
        .id();
    Some(entity)
}

fn render_mesh(chunk_mesh: ChunkMesh) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(chunk_mesh.positions),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(chunk_mesh.normals),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(chunk_mesh.colors),
    )
    .with_inserted_indices(Indices::U32(chunk_mesh.indices))
}

fn within_render_distance(chunk_position: ChunkPosition, center: ChunkPosition) -> bool {
    let distance = (chunk_position.0 - center.0).abs();
    distance.x <= RENDER_DISTANCE_IN_CHUNKS && distance.z <= RENDER_DISTANCE_IN_CHUNKS
}

fn chunks_touched_by(block_position: BlockPosition) -> impl Iterator<Item = ChunkPosition> {
    let (chunk_position, local_position) = block_position.split();
    let neighbors = [IVec3::X, IVec3::Y, IVec3::Z]
        .into_iter()
        .filter_map(move |axis| {
            let along_axis = (local_position * axis).element_sum();
            if along_axis == 0 {
                Some(ChunkPosition(chunk_position.0 - axis))
            } else if along_axis == CHUNK_SIZE - 1 {
                Some(ChunkPosition(chunk_position.0 + axis))
            } else {
                None
            }
        });
    std::iter::once(chunk_position).chain(neighbors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_inside_a_chunk_only_touches_that_chunk() {
        let touched: Vec<_> = chunks_touched_by(BlockPosition::new(5, 5, 5)).collect();
        assert_eq!(touched, vec![ChunkPosition::new(0, 0, 0)]);
    }

    #[test]
    fn a_block_on_a_border_also_touches_the_neighbor_across_it() {
        let touched: HashSet<_> = chunks_touched_by(BlockPosition::new(0, 5, 15)).collect();
        assert_eq!(
            touched,
            HashSet::from([
                ChunkPosition::new(0, 0, 0),
                ChunkPosition::new(-1, 0, 0),
                ChunkPosition::new(0, 0, 1),
            ])
        );
    }

    #[test]
    fn a_negative_border_block_touches_the_neighbor_below_it() {
        let touched: HashSet<_> = chunks_touched_by(BlockPosition::new(-16, -1, 3)).collect();
        assert_eq!(
            touched,
            HashSet::from([
                ChunkPosition::new(-1, -1, 0),
                ChunkPosition::new(-2, -1, 0),
                ChunkPosition::new(-1, 0, 0),
            ])
        );
    }

    #[test]
    fn render_distance_ignores_height_and_uses_the_square_around_the_center() {
        let center = ChunkPosition::new(2, 0, -3);
        assert!(within_render_distance(ChunkPosition::new(7, 9, -8), center));
        assert!(!within_render_distance(
            ChunkPosition::new(8, 0, -3),
            center
        ));
        assert!(!within_render_distance(ChunkPosition::new(2, 0, 3), center));
    }
}
