use glam::Vec3;
use crate::world::World;

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub bounding_box: AABB,
}

#[derive(Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_position(position: Vec3, half_width: f32, height: f32) -> Self {
        Self {
            min: Vec3::new(
                position.x - half_width,
                position.y,
                position.z - half_width,
            ),
            max: Vec3::new(
                position.x + half_width,
                position.y + height,
                position.z + half_width,
            ),
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
}

impl Player {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            on_ground: false,
            bounding_box: AABB::from_position(position, 0.3, 1.8),
        }
    }

    pub fn update_bounding_box(&mut self) {
        self.bounding_box = AABB::from_position(self.position, 0.3, 1.8);
    }

    pub fn apply_physics(&mut self, delta_time: f32, world: &World) {
        const GRAVITY: f32 = -25.0;
        const TERMINAL_VELOCITY: f32 = -50.0;

        // Apply gravity
        if !self.on_ground {
            self.velocity.y += GRAVITY * delta_time;
            if self.velocity.y < TERMINAL_VELOCITY {
                self.velocity.y = TERMINAL_VELOCITY;
            }
        }

        // Apply velocity with collision detection
        let new_position = self.position + self.velocity * delta_time;

        // Check Y collision (vertical)
        self.position.y = new_position.y;
        self.update_bounding_box();
        
        let y_collision = self.check_collision(world);
        if y_collision {
            if self.velocity.y < 0.0 {
                // Hit ground
                self.on_ground = true;
                self.velocity.y = 0.0;
                // Snap to block top
                let block_y = self.position.y.floor();
                self.position.y = block_y + 1.0;
            } else {
                // Hit ceiling
                self.velocity.y = 0.0;
                let block_y = (self.position.y + 1.8).ceil();
                self.position.y = block_y - 1.8;
            }
        } else {
            self.on_ground = false;
        }

        // Check X collision (horizontal)
        self.position.x = new_position.x;
        self.update_bounding_box();
        if self.check_collision(world) {
            self.position.x = self.position.x.round();
            if self.check_collision(world) {
                // Revert X movement
                self.position.x -= self.velocity.x * delta_time;
            }
        }

        // Check Z collision (horizontal)
        self.position.z = new_position.z;
        self.update_bounding_box();
        if self.check_collision(world) {
            self.position.z = self.position.z.round();
            if self.check_collision(world) {
                // Revert Z movement
                self.position.z -= self.velocity.z * delta_time;
            }
        }

        self.update_bounding_box();
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = 8.0;
            self.on_ground = false;
        }
    }

    fn check_collision(&self, world: &World) -> bool {
        // Check all blocks that could intersect with the player's bounding box
        let min_x = self.bounding_box.min.x.floor() as i32;
        let max_x = self.bounding_box.max.x.ceil() as i32;
        let min_y = self.bounding_box.min.y.floor() as i32;
        let max_y = self.bounding_box.max.y.ceil() as i32;
        let min_z = self.bounding_box.min.z.floor() as i32;
        let max_z = self.bounding_box.max.z.ceil() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if let Some(block_type) = world.get_block_at(x, y, z) {
                        if block_type.is_solid() {
                            let block_aabb = AABB::new(
                                Vec3::new(x as f32, y as f32, z as f32),
                                Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                            );
                            if self.bounding_box.intersects(&block_aabb) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
