use glam::{IVec3, Vec2, Vec3};

use crate::position::BlockPosition;
use crate::world::World;

pub const WALK_SPEED: f32 = 4.5;
pub const GRAVITY: f32 = 25.0;
pub const JUMP_HEIGHT: f32 = 1.2;
pub const MAX_FALL_SPEED: f32 = 40.0;
const CONTACT_EPSILON: f32 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub position: Vec3,
    pub velocity: Vec3,
    pub half_width: f32,
    pub height: f32,
    pub grounded: bool,
}

impl Body {
    pub fn standing_at(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            half_width: 0.3,
            height: 1.8,
            grounded: false,
        }
    }

    pub fn overlaps_block(&self, block_position: BlockPosition) -> bool {
        let (min, max) = self.bounds();
        let block_min = block_position.min_corner();
        let block_max = block_min + Vec3::ONE;
        min.cmplt(block_max).all() && max.cmpgt(block_min).all()
    }

    fn bounds(&self) -> (Vec3, Vec3) {
        let half_extent = Vec3::new(self.half_width, 0.0, self.half_width);
        let min = self.position - half_extent;
        let max = self.position + half_extent + Vec3::new(0.0, self.height, 0.0);
        (min, max)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MovementInput {
    pub horizontal: Vec2,
    pub jump: bool,
}

pub fn jump_speed() -> f32 {
    (2.0 * GRAVITY * JUMP_HEIGHT).sqrt()
}

pub fn step_body(world: &World, body: Body, movement: MovementInput, delta_seconds: f32) -> Body {
    let mut body = body;
    let horizontal = movement.horizontal.clamp_length_max(1.0) * WALK_SPEED;
    body.velocity.x = horizontal.x;
    body.velocity.z = horizontal.y;
    if movement.jump && body.grounded {
        body.velocity.y = jump_speed();
    }
    body.velocity.y = (body.velocity.y - GRAVITY * delta_seconds).max(-MAX_FALL_SPEED);

    let displacement = body.velocity * delta_seconds;
    body.grounded = false;
    body = sweep_axis(world, body, Axis::Y, displacement.y);
    body = sweep_axis(world, body, Axis::X, displacement.x);
    sweep_axis(world, body, Axis::Z, displacement.z)
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

fn sweep_axis(world: &World, mut body: Body, axis: Axis, distance: f32) -> Body {
    if distance == 0.0 {
        return body;
    }
    body.position[axis.index()] += distance;
    let Some(blocking) = first_overlapping_solid_block(world, &body) else {
        return body;
    };
    let block_min = blocking.min_corner()[axis.index()];
    let block_max = block_min + 1.0;
    let extent_below_center = match axis {
        Axis::Y => 0.0,
        Axis::X | Axis::Z => body.half_width,
    };
    let extent_above_center = match axis {
        Axis::Y => body.height,
        Axis::X | Axis::Z => body.half_width,
    };
    body.position[axis.index()] = if distance > 0.0 {
        block_min - extent_above_center - CONTACT_EPSILON
    } else {
        block_max + extent_below_center + CONTACT_EPSILON
    };
    body.velocity[axis.index()] = 0.0;
    if matches!(axis, Axis::Y) && distance < 0.0 {
        body.grounded = true;
    }
    body
}

fn first_overlapping_solid_block(world: &World, body: &Body) -> Option<BlockPosition> {
    let (min, max) = body.bounds();
    let first = BlockPosition::containing(min).0;
    let last = BlockPosition::containing(max - Vec3::splat(CONTACT_EPSILON)).0;
    for x in first.x..=last.x {
        for y in first.y..=last.y {
            for z in first.z..=last.z {
                let block_position = BlockPosition(IVec3::new(x, y, z));
                if world.is_solid_at(block_position) {
                    return Some(block_position);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;

    const DELTA_SECONDS: f32 = 1.0 / 60.0;

    fn floor_world() -> World {
        let mut world = World::default();
        for x in -8..8 {
            for z in -8..8 {
                world.set_block(BlockPosition::new(x, 0, z), Block::Grass);
            }
        }
        world
    }

    fn simulate(world: &World, mut body: Body, movement: MovementInput, steps: usize) -> Body {
        for _ in 0..steps {
            body = step_body(world, body, movement, DELTA_SECONDS);
        }
        body
    }

    #[test]
    fn a_body_dropped_from_above_lands_on_the_surface_and_is_grounded() {
        let world = floor_world();
        let body = simulate(
            &world,
            Body::standing_at(Vec3::new(0.5, 5.0, 0.5)),
            MovementInput::default(),
            120,
        );
        assert!(body.grounded);
        assert_eq!(body.velocity.y, 0.0);
        assert!(
            (body.position.y - 1.0).abs() < 0.01,
            "feet at {}",
            body.position.y
        );
    }

    #[test]
    fn a_body_falls_forever_where_there_is_no_ground() {
        let body = simulate(
            &World::default(),
            Body::standing_at(Vec3::new(0.5, 5.0, 0.5)),
            MovementInput::default(),
            60,
        );
        assert!(!body.grounded);
        assert!(body.position.y < 0.0);
        assert!(body.velocity.y < 0.0);
    }

    #[test]
    fn a_body_cannot_walk_through_a_solid_wall() {
        let mut world = floor_world();
        for y in 1..4 {
            for z in -8..8 {
                world.set_block(BlockPosition::new(3, y, z), Block::Brick);
            }
        }
        let start = simulate(
            &world,
            Body::standing_at(Vec3::new(0.5, 1.0, 0.5)),
            MovementInput::default(),
            10,
        );
        let body = simulate(
            &world,
            start,
            MovementInput {
                horizontal: Vec2::new(1.0, 0.0),
                jump: false,
            },
            120,
        );
        assert!(body.position.x < 3.0 - body.half_width);
        assert!(body.position.x > 2.5, "stopped at {}", body.position.x);
        assert_eq!(body.velocity.x, 0.0);
        assert!(body.grounded);
    }

    #[test]
    fn walking_moves_at_walk_speed_along_the_requested_direction() {
        let world = floor_world();
        let start = simulate(
            &world,
            Body::standing_at(Vec3::new(0.5, 1.0, 0.5)),
            MovementInput::default(),
            10,
        );
        let body = simulate(
            &world,
            start,
            MovementInput {
                horizontal: Vec2::new(0.0, -1.0),
                jump: false,
            },
            60,
        );
        assert!(
            (body.position.z - (0.5 - WALK_SPEED)).abs() < 0.05,
            "{}",
            body.position.z
        );
        assert_eq!(body.position.x, 0.5);
    }

    #[test]
    fn jumping_is_only_possible_while_grounded() {
        let world = floor_world();
        let grounded = simulate(
            &world,
            Body::standing_at(Vec3::new(0.5, 1.0, 0.5)),
            MovementInput::default(),
            10,
        );
        let jump = MovementInput {
            horizontal: Vec2::ZERO,
            jump: true,
        };
        let airborne = step_body(&world, grounded, jump, DELTA_SECONDS);
        assert!(airborne.velocity.y > 0.0);
        assert!(!airborne.grounded);

        let still_airborne = step_body(&world, airborne, jump, DELTA_SECONDS);
        assert!(still_airborne.velocity.y < airborne.velocity.y);
    }

    #[test]
    fn a_jump_peaks_about_one_block_and_a_bit_above_the_ground() {
        let world = floor_world();
        let grounded = simulate(
            &world,
            Body::standing_at(Vec3::new(0.5, 1.0, 0.5)),
            MovementInput::default(),
            10,
        );
        let mut body = step_body(
            &world,
            grounded,
            MovementInput {
                horizontal: Vec2::ZERO,
                jump: true,
            },
            DELTA_SECONDS,
        );
        let mut peak = body.position.y;
        for _ in 0..120 {
            body = step_body(&world, body, MovementInput::default(), DELTA_SECONDS);
            peak = peak.max(body.position.y);
        }
        let jump_height = peak - 1.0;
        assert!(
            (JUMP_HEIGHT - 0.15..=JUMP_HEIGHT + 0.05).contains(&jump_height),
            "{jump_height}"
        );
        assert!(body.grounded);
    }

    #[test]
    fn a_body_bumps_its_head_on_a_ceiling() {
        let mut world = floor_world();
        for x in -8..8 {
            for z in -8..8 {
                world.set_block(BlockPosition::new(x, 3, z), Block::Stone);
            }
        }
        let grounded = simulate(
            &world,
            Body::standing_at(Vec3::new(0.5, 1.0, 0.5)),
            MovementInput::default(),
            10,
        );
        let mut body = step_body(
            &world,
            grounded,
            MovementInput {
                horizontal: Vec2::ZERO,
                jump: true,
            },
            DELTA_SECONDS,
        );
        for _ in 0..30 {
            body = step_body(&world, body, MovementInput::default(), DELTA_SECONDS);
            assert!(body.position.y + body.height <= 3.0);
        }
    }

    #[test]
    fn overlap_with_a_block_uses_the_feet_centered_bounds() {
        let body = Body::standing_at(Vec3::new(0.5, 1.0, 0.5));
        assert!(body.overlaps_block(BlockPosition::new(0, 1, 0)));
        assert!(body.overlaps_block(BlockPosition::new(0, 2, 0)));
        assert!(!body.overlaps_block(BlockPosition::new(0, 0, 0)));
        assert!(!body.overlaps_block(BlockPosition::new(1, 1, 0)));
    }
}
