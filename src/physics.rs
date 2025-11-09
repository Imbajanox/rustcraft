use glam::Vec3;
use crate::world::World;

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub bounding_box: Aabb,
}

#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
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

    pub fn intersects(&self, other: &Aabb) -> bool {
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
            bounding_box: Aabb::from_position(position, 0.3, 1.8),
        }
    }

    pub fn update_bounding_box(&mut self) {
        self.bounding_box = Aabb::from_position(self.position, 0.3, 1.8);
    }

    pub fn apply_physics(&mut self, delta_time: f32, world: &World) {
        const GRAVITY: f32 = -25.0;
        const TERMINAL_VELOCITY: f32 = -50.0;
        const EPSILON: f32 = 0.001;
        const STEP_HEIGHT: f32 = 0.5; // Allow stepping up small blocks

        // Apply gravity
        if !self.on_ground {
            self.velocity.y += GRAVITY * delta_time;
            if self.velocity.y < TERMINAL_VELOCITY {
                self.velocity.y = TERMINAL_VELOCITY;
            }
        }

        // Calculate target position
        let desired_position = self.position + self.velocity * delta_time;
        
        // Handle Y axis (vertical) separately with improved collision
        let old_y = self.position.y;
        self.position.y = desired_position.y;
        self.update_bounding_box();
        
        if self.check_collision(world) {
            if self.velocity.y < 0.0 {
                // Falling - snap to ground smoothly
                self.on_ground = true;
                self.velocity.y = 0.0;
                // Find the exact ground position
                // Player's feet (min.y) need to be just above the block
                // If bounding_box.min.y is inside a block at y, position should be y+1
                let ground_block_y = self.bounding_box.min.y.floor();
                self.position.y = ground_block_y + 1.0;
            } else {
                // Rising - hit ceiling
                self.velocity.y = 0.0;
                // If bounding_box.max.y hits a block at y, position should be y - 1.8
                let ceiling_block_y = self.bounding_box.max.y.floor();
                self.position.y = ceiling_block_y - 1.8 - EPSILON;
            }
            self.update_bounding_box();
        } else {
            self.on_ground = false;
        }

        // Handle X and Z (horizontal) with improved sliding and step detection
        let old_x = self.position.x;
        let old_z = self.position.z;
        
        // Try full movement
        self.position.x = desired_position.x;
        self.position.z = desired_position.z;
        self.update_bounding_box();
        
        if self.check_collision(world) {
            // Try stepping up if on ground
            if self.on_ground {
                let step_y = old_y + STEP_HEIGHT;
                self.position.y = step_y;
                self.update_bounding_box();
                
                if !self.check_collision(world) {
                    // Successfully stepped up, keep the new position
                    return;
                }
                // Step failed, revert y
                self.position.y = old_y;
            }
            
            // Full movement failed, try sliding along walls
            
            // Try moving only in X direction
            self.position.x = desired_position.x;
            self.position.z = old_z;
            self.update_bounding_box();
            
            if self.check_collision(world) {
                // X movement blocked, reset X
                self.position.x = old_x;
            }
            
            // Try moving only in Z direction
            self.position.z = desired_position.z;
            self.update_bounding_box();
            
            if self.check_collision(world) {
                // Z movement blocked, reset Z
                self.position.z = old_z;
            }
            
            self.update_bounding_box();
        }
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = 8.0;
            self.on_ground = false;
        }
    }

    // Check if player can fit through a space at a given position
    #[allow(dead_code)]
    pub fn can_fit(&self, position: Vec3, world: &World) -> bool {
        let test_box = Aabb::from_position(position, 0.3, 1.8);
        
        let min_x = test_box.min.x.floor() as i32;
        let max_x = test_box.max.x.ceil() as i32;
        let min_y = test_box.min.y.floor() as i32;
        let max_y = test_box.max.y.ceil() as i32;
        let min_z = test_box.min.z.floor() as i32;
        let max_z = test_box.max.z.ceil() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if let Some(block_type) = world.get_block_at(x, y, z) {
                        if block_type.is_solid() {
                            let block_aabb = Aabb::new(
                                Vec3::new(x as f32, y as f32, z as f32),
                                Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                            );
                            if test_box.intersects(&block_aabb) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    fn check_collision(&self, world: &World) -> bool {
        const MARGIN: f32 = 0.001; // Small margin to prevent floating point issues
        
        // Check all blocks that could intersect with the player's bounding box
        // Add small margin to prevent edge cases
        let min_x = (self.bounding_box.min.x - MARGIN).floor() as i32;
        let max_x = (self.bounding_box.max.x + MARGIN).ceil() as i32;
        let min_y = (self.bounding_box.min.y - MARGIN).floor() as i32;
        let max_y = (self.bounding_box.max.y + MARGIN).ceil() as i32;
        let min_z = (self.bounding_box.min.z - MARGIN).floor() as i32;
        let max_z = (self.bounding_box.max.z + MARGIN).ceil() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if let Some(block_type) = world.get_block_at(x, y, z) {
                        if block_type.is_solid() {
                            let block_aabb = Aabb::new(
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
