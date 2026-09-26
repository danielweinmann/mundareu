use glam::{IVec3, Vec3};

use crate::position::BlockPosition;
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaycastHit {
    pub block_position: BlockPosition,
    pub face_normal: IVec3,
}

pub fn raycast(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<RaycastHit> {
    let direction = direction.try_normalize()?;
    let mut current = BlockPosition::containing(origin).0;
    let step = direction.signum().as_ivec3();
    let mut distance_to_next_boundary = Vec3::ZERO;
    let mut distance_per_block = Vec3::ZERO;
    for axis in 0..3 {
        if direction[axis] == 0.0 {
            distance_to_next_boundary[axis] = f32::INFINITY;
            distance_per_block[axis] = f32::INFINITY;
        } else {
            let next_boundary = if direction[axis] > 0.0 {
                current[axis] as f32 + 1.0
            } else {
                current[axis] as f32
            };
            distance_to_next_boundary[axis] = (next_boundary - origin[axis]) / direction[axis];
            distance_per_block[axis] = 1.0 / direction[axis].abs();
        }
    }

    loop {
        let axis = smallest_axis(distance_to_next_boundary);
        let travelled = distance_to_next_boundary[axis];
        if travelled > max_distance {
            return None;
        }
        current[axis] += step[axis];
        distance_to_next_boundary[axis] += distance_per_block[axis];
        let block_position = BlockPosition(current);
        if world.is_solid_at(block_position) {
            let mut face_normal = IVec3::ZERO;
            face_normal[axis] = -step[axis];
            return Some(RaycastHit {
                block_position,
                face_normal,
            });
        }
    }
}

fn smallest_axis(distances: Vec3) -> usize {
    if distances.x <= distances.y && distances.x <= distances.z {
        0
    } else if distances.y <= distances.z {
        1
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::Block;

    fn world_with_block(block_position: BlockPosition) -> World {
        let mut world = World::default();
        world.set_block(block_position, Block::Stone);
        world
    }

    #[test]
    fn an_axis_aligned_ray_hits_the_facing_side_of_the_block() {
        let world = world_with_block(BlockPosition::new(5, 0, 0));
        let hit = raycast(&world, Vec3::new(0.5, 0.5, 0.5), Vec3::X, 10.0);
        assert_eq!(
            hit,
            Some(RaycastHit {
                block_position: BlockPosition::new(5, 0, 0),
                face_normal: IVec3::NEG_X,
            })
        );
    }

    #[test]
    fn a_downward_ray_hits_the_top_face() {
        let world = world_with_block(BlockPosition::new(0, -3, 0));
        let hit = raycast(&world, Vec3::new(0.5, 2.5, 0.5), Vec3::NEG_Y, 10.0);
        assert_eq!(
            hit,
            Some(RaycastHit {
                block_position: BlockPosition::new(0, -3, 0),
                face_normal: IVec3::Y,
            })
        );
    }

    #[test]
    fn a_diagonal_ray_hits_the_block_on_the_face_it_crosses_first() {
        let world = world_with_block(BlockPosition::new(3, 0, 3));
        let hit = raycast(
            &world,
            Vec3::new(0.5, 0.5, 0.2),
            Vec3::new(1.0, 0.0, 1.0),
            10.0,
        );
        assert_eq!(
            hit,
            Some(RaycastHit {
                block_position: BlockPosition::new(3, 0, 3),
                face_normal: IVec3::NEG_Z,
            })
        );
    }

    #[test]
    fn nothing_within_range_yields_no_hit() {
        let world = world_with_block(BlockPosition::new(20, 0, 0));
        assert_eq!(
            raycast(&world, Vec3::new(0.5, 0.5, 0.5), Vec3::X, 10.0),
            None
        );
        assert_eq!(
            raycast(&World::default(), Vec3::ZERO, Vec3::NEG_Y, 100.0),
            None
        );
    }

    #[test]
    fn a_zero_direction_yields_no_hit() {
        let world = world_with_block(BlockPosition::new(1, 0, 0));
        assert_eq!(
            raycast(&world, Vec3::new(0.5, 0.5, 0.5), Vec3::ZERO, 10.0),
            None
        );
    }

    #[test]
    fn a_ray_from_negative_coordinates_traverses_correctly() {
        let world = world_with_block(BlockPosition::new(-1, -1, -1));
        let hit = raycast(&world, Vec3::new(-0.5, -0.5, 2.5), Vec3::NEG_Z, 10.0);
        assert_eq!(
            hit,
            Some(RaycastHit {
                block_position: BlockPosition::new(-1, -1, -1),
                face_normal: IVec3::Z,
            })
        );
    }
}
