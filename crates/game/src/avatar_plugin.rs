use bevy::prelude::*;
use mundareu_world::{Body, MovementInput, PlayerState, step_body};

use crate::GameSystems;
use crate::input_plugin::PlayerIntent;
use crate::world_plugin::Terrain;

const CAMERA_DISTANCE: f32 = 6.0;
const CAMERA_TARGET_HEIGHT: f32 = 1.4;
const STARTING_PITCH: f32 = 0.35;
const MIN_PITCH: f32 = -0.3;
const MAX_PITCH: f32 = 1.3;
const FALLBACK_SPAWN_HEIGHT: f32 = 24.0;

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct StartingPlayer(pub Option<PlayerState>);

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Avatar {
    pub body: Body,
    previous_position: Vec3,
    pub facing_yaw: f32,
}

impl Avatar {
    fn drawn_position(&self, overstep_fraction: f32) -> Vec3 {
        self.previous_position
            .lerp(self.body.position, overstep_fraction)
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_avatar_and_camera)
        .add_systems(FixedUpdate, step_avatar)
        .add_systems(
            Update,
            (
                sync_avatar_transform.in_set(GameSystems::Avatar),
                orbit_camera.in_set(GameSystems::Camera),
            ),
        );
}

fn spawn_avatar_and_camera(
    mut commands: Commands,
    starting_player: Res<StartingPlayer>,
    terrain: Res<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player = starting_player
        .0
        .unwrap_or_else(|| fresh_player_state(&terrain));
    let skin = materials.add(Color::srgb(0.96, 0.78, 0.62));
    let shirt = materials.add(Color::srgb(0.98, 0.62, 0.30));
    let shorts = materials.add(Color::srgb(0.32, 0.50, 0.85));
    let mut part = |size: Vec3, center: Vec3, material: &Handle<StandardMaterial>| {
        (
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(center),
        )
    };
    commands.spawn((
        Avatar {
            body: Body::standing_at(player.position),
            previous_position: player.position,
            facing_yaw: player.yaw,
        },
        Transform::from_translation(player.position),
        Visibility::default(),
        children![
            part(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.0, 1.55, 0.0), &skin),
            part(Vec3::new(0.6, 0.7, 0.35), Vec3::new(0.0, 0.95, 0.0), &shirt),
            part(
                Vec3::new(0.25, 0.6, 0.3),
                Vec3::new(-0.16, 0.3, 0.0),
                &shorts
            ),
            part(
                Vec3::new(0.25, 0.6, 0.3),
                Vec3::new(0.16, 0.3, 0.0),
                &shorts
            ),
            part(
                Vec3::new(0.2, 0.65, 0.25),
                Vec3::new(-0.42, 0.95, 0.0),
                &skin
            ),
            part(
                Vec3::new(0.2, 0.65, 0.25),
                Vec3::new(0.42, 0.95, 0.0),
                &skin
            ),
        ],
    ));
    commands.spawn((
        Camera3d::default(),
        OrbitCamera {
            yaw: player.yaw,
            pitch: STARTING_PITCH,
        },
        Transform::default(),
    ));
}

fn fresh_player_state(terrain: &Terrain) -> PlayerState {
    let surface_y = terrain
        .0
        .highest_solid_y(0, 0, 63)
        .map_or(FALLBACK_SPAWN_HEIGHT, |y| (y + 1) as f32);
    PlayerState {
        position: Vec3::new(0.5, surface_y, 0.5),
        yaw: 0.0,
    }
}

fn step_avatar(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut intent: ResMut<PlayerIntent>,
    mut avatar: Single<&mut Avatar>,
    camera: Single<&OrbitCamera>,
) {
    let horizontal = world_movement(camera.yaw, intent.movement);
    let movement = MovementInput {
        horizontal,
        jump: intent.jump,
    };
    intent.jump = false;
    avatar.previous_position = avatar.body.position;
    avatar.body = step_body(&terrain.0, avatar.body, movement, time.delta_secs());
    if horizontal != Vec2::ZERO {
        avatar.facing_yaw = yaw_facing(horizontal);
    }
}

fn sync_avatar_transform(
    fixed_time: Res<Time<Fixed>>,
    mut avatar: Single<(&Avatar, &mut Transform)>,
) {
    let (avatar, transform) = &mut *avatar;
    transform.translation = avatar.drawn_position(fixed_time.overstep_fraction());
    transform.rotation = Quat::from_rotation_y(avatar.facing_yaw);
}

fn orbit_camera(
    fixed_time: Res<Time<Fixed>>,
    mut intent: ResMut<PlayerIntent>,
    avatar: Single<&Avatar>,
    mut camera: Single<(&mut OrbitCamera, &mut Transform)>,
) {
    let (orbit, transform) = &mut *camera;
    orbit.yaw -= intent.look.x;
    orbit.pitch = (orbit.pitch + intent.look.y).clamp(MIN_PITCH, MAX_PITCH);
    intent.look = Vec2::ZERO;
    let target =
        avatar.drawn_position(fixed_time.overstep_fraction()) + Vec3::Y * CAMERA_TARGET_HEIGHT;
    let forward = camera_forward(orbit.yaw, orbit.pitch);
    **transform =
        Transform::from_translation(target - forward * CAMERA_DISTANCE).looking_at(target, Vec3::Y);
}

fn camera_forward(yaw: f32, pitch: f32) -> Vec3 {
    let flat = flat_forward(yaw);
    Vec3::new(flat.x * pitch.cos(), -pitch.sin(), flat.y * pitch.cos())
}

fn flat_forward(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

fn world_movement(camera_yaw: f32, camera_relative: Vec2) -> Vec2 {
    let forward = flat_forward(camera_yaw);
    let right = Vec2::new(-forward.y, forward.x);
    right * camera_relative.x + forward * camera_relative.y
}

fn yaw_facing(direction: Vec2) -> f32 {
    (-direction.x).atan2(-direction.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: Vec2, expected: Vec2) {
        assert!(
            actual.distance(expected) < 1e-5,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn with_zero_yaw_forward_is_negative_z_and_right_is_positive_x() {
        assert_close(
            world_movement(0.0, Vec2::new(0.0, 1.0)),
            Vec2::new(0.0, -1.0),
        );
        assert_close(
            world_movement(0.0, Vec2::new(1.0, 0.0)),
            Vec2::new(1.0, 0.0),
        );
    }

    #[test]
    fn turning_the_camera_a_quarter_left_turns_forward_towards_negative_x() {
        let quarter_turn = std::f32::consts::FRAC_PI_2;
        assert_close(
            world_movement(quarter_turn, Vec2::new(0.0, 1.0)),
            Vec2::new(-1.0, 0.0),
        );
        assert_close(
            world_movement(quarter_turn, Vec2::new(1.0, 0.0)),
            Vec2::new(0.0, -1.0),
        );
    }

    #[test]
    fn the_facing_yaw_reproduces_the_movement_direction() {
        for direction in [
            Vec2::new(0.0, -1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(-0.7, 0.7),
        ] {
            let yaw = yaw_facing(direction);
            assert_close(flat_forward(yaw), direction.normalize());
        }
    }

    #[test]
    fn the_avatar_is_drawn_between_its_last_two_fixed_steps() {
        let avatar = Avatar {
            body: Body::standing_at(Vec3::new(2.0, 10.0, -4.0)),
            previous_position: Vec3::new(1.0, 10.0, -2.0),
            facing_yaw: 0.0,
        };
        assert_eq!(avatar.drawn_position(0.0), Vec3::new(1.0, 10.0, -2.0));
        assert_eq!(avatar.drawn_position(0.25), Vec3::new(1.25, 10.0, -2.5));
        assert_eq!(avatar.drawn_position(1.0), Vec3::new(2.0, 10.0, -4.0));
    }

    #[test]
    fn a_positive_pitch_looks_down_from_above() {
        let forward = camera_forward(0.0, 0.5);
        assert!(forward.y < 0.0);
        assert!(forward.z < 0.0);
        assert!((forward.length() - 1.0).abs() < 1e-5);
    }
}
