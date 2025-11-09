use glam::Vec3;
use crate::world::World;

pub struct RaycastResult {
    pub hit: bool,
    pub position: Option<(i32, i32, i32)>,
    pub normal: Option<(i32, i32, i32)>,
}

pub fn raycast(origin: Vec3, direction: Vec3, max_distance: f32, world: &World) -> RaycastResult {
    let step = 0.1;
    let max_steps = (max_distance / step) as i32;

    let mut current = origin;
    let mut previous = origin;

    for _ in 0..max_steps {
        current += direction * step;

        let x = current.x.floor() as i32;
        let y = current.y.floor() as i32;
        let z = current.z.floor() as i32;

        if let Some(block) = world.get_block_at(x, y, z) {
            if block.is_solid() {
                // Calculate the normal based on which face was hit
                let prev_x = previous.x.floor() as i32;
                let prev_y = previous.y.floor() as i32;
                let prev_z = previous.z.floor() as i32;

                let normal = (
                    prev_x - x,
                    prev_y - y,
                    prev_z - z,
                );

                return RaycastResult {
                    hit: true,
                    position: Some((x, y, z)),
                    normal: Some(normal),
                };
            }
        }

        previous = current;
    }

    RaycastResult {
        hit: false,
        position: None,
        normal: None,
    }
}
