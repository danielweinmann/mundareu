use bevy::prelude::*;
use mundareu_world::{Block, raycast};

use crate::GameSystems;
use crate::avatar_plugin::{Avatar, OrbitCamera};
use crate::input_plugin::PlayerIntent;
use crate::world_plugin::{BlockChanged, Terrain};

pub const PALETTE: [Block; 6] = [
    Block::Grass,
    Block::Dirt,
    Block::Stone,
    Block::Sand,
    Block::Wood,
    Block::Brick,
];
const REACH_IN_BLOCKS: f32 = 12.0;
const RETICLE_SIZE_IN_PIXELS: f32 = 10.0;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectedPaletteIndex(pub usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<SelectedPaletteIndex>()
        .add_systems(Startup, spawn_reticle)
        .add_systems(
            Update,
            (select_palette_block, edit_blocks)
                .chain()
                .in_set(GameSystems::Building),
        );
}

fn spawn_reticle(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(RETICLE_SIZE_IN_PIXELS),
            height: px(RETICLE_SIZE_IN_PIXELS),
            margin: UiRect::all(px(-RETICLE_SIZE_IN_PIXELS / 2.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
    ));
}

fn select_palette_block(
    mut intent: ResMut<PlayerIntent>,
    mut selected: ResMut<SelectedPaletteIndex>,
) {
    if let Some(index) = intent.select_palette_index.take()
        && index < PALETTE.len()
    {
        selected.0 = index;
    }
}

fn edit_blocks(
    mut intent: ResMut<PlayerIntent>,
    mut terrain: ResMut<Terrain>,
    mut block_changes: MessageWriter<BlockChanged>,
    selected: Res<SelectedPaletteIndex>,
    avatar: Single<&Avatar>,
    camera: Single<&Transform, With<OrbitCamera>>,
) {
    let break_block = std::mem::take(&mut intent.break_block);
    let place_block = std::mem::take(&mut intent.place_block);
    if !break_block && !place_block {
        return;
    }
    let Some(hit) = raycast(
        &terrain.0,
        camera.translation,
        camera.forward().as_vec3(),
        REACH_IN_BLOCKS,
    ) else {
        return;
    };
    if break_block {
        terrain.0.set_block(hit.block_position, Block::Air);
        block_changes.write(BlockChanged {
            block_position: hit.block_position,
        });
    } else {
        let target = hit.block_position.offset(hit.face_normal);
        if !avatar.body.overlaps_block(target) {
            terrain.0.set_block(target, PALETTE[selected.0]);
            block_changes.write(BlockChanged {
                block_position: target,
            });
        }
    }
}
