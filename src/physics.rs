use glam::Vec3;
use crate::world::World;

// Aabb and Player struct remain unchanged
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

// --- START: Fixed Player Implementation ---

impl Player {
    // ⚠️ New, unified constant for actual collision size (0.3 for 0.6 total width)
    const COLLISION_HALF_WIDTH: f32 = 0.3; 
    const PLAYER_HEIGHT: f32 = 1.8;

    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            on_ground: false,
            // Use the unified constant
            bounding_box: Aabb::from_position(position, Self::COLLISION_HALF_WIDTH, Self::PLAYER_HEIGHT),
        }
    }

    pub fn update_bounding_box(&mut self) {
        // Use the unified constant
        self.bounding_box = Aabb::from_position(self.position, Self::COLLISION_HALF_WIDTH, Self::PLAYER_HEIGHT);
    }

    pub fn apply_physics(&mut self, delta_time: f32, world: &World) {
        const GRAVITY: f32 = -25.0;
        const TERMINAL_VELOCITY: f32 = -50.0;
        const EPSILON: f32 = 0.001;
        // Use the unified constant for horizontal calculations
        const HALF_WIDTH: f32 = Player::COLLISION_HALF_WIDTH; 
        const HEIGHT: f32 = Player::PLAYER_HEIGHT;
        const STEP_HEIGHT: f32 = 0.6; // Max height the player can climb

        let prev_position = self.position;
        let prev_feet_y = prev_position.y;

        // --- 1. Optimized Support Check when on_ground (Retaining your previous robust fix) ---
        if self.on_ground {
            let support_probe = 0.05_f32; // small probe distance
            let feet_y = self.position.y;
            let mut supported = false;

            // Define the bounding box of the area to check for support
            let support_aabb = Aabb {
                min: Vec3::new(
                    self.bounding_box.min.x,
                    feet_y - support_probe,
                    self.bounding_box.min.z,
                ),
                max: Vec3::new(
                    self.bounding_box.max.x,
                    feet_y + support_probe,
                    self.bounding_box.max.z,
                ),
            };

            let min_x = support_aabb.min.x.floor() as i32;
            let max_x = support_aabb.max.x.ceil() as i32;
            let min_z = support_aabb.min.z.floor() as i32;
            let max_z = support_aabb.max.z.ceil() as i32;
            let check_y = (feet_y - EPSILON).floor() as i32;

            'support_loop: for x in min_x..=max_x {
                for z in min_z..=max_z {
                    let check_for_support = |cy: i32| -> bool {
                        if let Some(block_type) = world.get_block_at(x, cy, z) {
                            if block_type.is_solid() {
                                let block_top = cy as f32 + 1.0;
                                // 1. Check if the block's top is at the right height (near feet_y)
                                if (block_top - feet_y).abs() <= support_probe + EPSILON {
                                    // 2. Check if the block intersects the player's support AABB horizontally
                                    let block_aabb = Aabb::new(
                                        Vec3::new(x as f32, cy as f32, z as f32),
                                        Vec3::new((x + 1) as f32, (cy + 1) as f32, (z + 1) as f32),
                                    );
                                    // Use explicit XZ intersection check
                                    if support_aabb.min.x < block_aabb.max.x 
                                        && support_aabb.max.x > block_aabb.min.x
                                        && support_aabb.min.z < block_aabb.max.z
                                        && support_aabb.max.z > block_aabb.min.z 
                                    {
                                        return true;
                                    }
                                }
                            }
                        }
                        false
                    };

                    if check_for_support(check_y) || check_for_support(check_y - 1) {
                        supported = true;
                        break 'support_loop;
                    }
                }
            }

            if !supported {
                self.on_ground = false;
            }
        }

        // Apply gravity
        if !self.on_ground {
            self.velocity.y += GRAVITY * delta_time;
            self.velocity.y = self.velocity.y.max(TERMINAL_VELOCITY);
        }

        let desired_position = self.position + self.velocity * delta_time;

        // --- 2. Vertical Sweep/Tunneling Prevention ---
        if self.velocity.y < 0.0 {
            let desired_feet_y = desired_position.y;
            let mut landing_y: Option<i32> = None;
            let mut landed = false;

            // Calculate swept X/Z range using the correct HALF_WIDTH (0.3)
            let prev_bb = Aabb::from_position(prev_position, HALF_WIDTH, HEIGHT);
            let desired_bb_proj = Aabb::from_position(Vec3::new(desired_position.x, prev_position.y, desired_position.z), HALF_WIDTH, HEIGHT);
            
            let swept_min_x = prev_bb.min.x.min(desired_bb_proj.min.x).floor() as i32;
            let swept_max_x = prev_bb.max.x.max(desired_bb_proj.max.x).ceil() as i32;
            let swept_min_z = prev_bb.min.z.min(desired_bb_proj.min.z).floor() as i32;
            let swept_max_z = prev_bb.max.z.max(desired_bb_proj.max.z).ceil() as i32;
            
            let check_min_y = (desired_feet_y - EPSILON).floor() as i32; 
            let check_max_y = (prev_feet_y + EPSILON).ceil() as i32;

            for y in check_min_y..=check_max_y {
                let block_top = y as f32 + 1.0;
                if block_top > prev_feet_y + EPSILON || block_top < desired_feet_y - EPSILON {
                    continue;
                }

                'xz_loop: for x in swept_min_x..=swept_max_x {
                    for z in swept_min_z..=swept_max_z {
                        if let Some(block_type) = world.get_block_at(x, y, z) {
                            if block_type.is_solid() {
                                // ⚠️ Ensure block AABB intersects the player AABB horizontally
                                let block_aabb = Aabb::new(
                                    Vec3::new(x as f32, y as f32, z as f32),
                                    Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                                );
                                
                                // Create a projected AABB for the player at the block's top height
                                let projected_bb = Aabb::from_position(
                                    Vec3::new(desired_position.x, block_top, desired_position.z), 
                                    HALF_WIDTH, 
                                    HEIGHT
                                );

                                if projected_bb.min.x < block_aabb.max.x 
                                    && projected_bb.max.x > block_aabb.min.x
                                    && projected_bb.min.z < block_aabb.max.z
                                    && projected_bb.max.z > block_aabb.min.z 
                                {
                                    landing_y = Some(y);
                                    landed = true;
                                    break 'xz_loop;
                                }
                            }
                        }
                    }
                }

                if landed {
                    break;
                }
            }

            if let Some(by) = landing_y {
                // Snap feet on top of block
                self.position.y = by as f32 + 1.0 + EPSILON;
                self.velocity.y = 0.0;
                self.on_ground = true;
            } else {
                self.position.y = desired_position.y;
            }
        } else {
            // Rising or stationary: apply desired Y
            self.position.y = desired_position.y;
            self.on_ground = self.velocity.y == 0.0 && self.on_ground; 
        }

        self.update_bounding_box();

        // --- 3. Vertical Collision Fallback/Ceiling Check (Unchanged, relies on check_collision) ---
        if self.check_collision(world) {
            if self.velocity.y > 0.0 {
                // Rising: hit ceiling. 
                self.velocity.y = 0.0;
                let mut lowest_block_y_above: Option<i32> = None;
                
                let min_x = self.bounding_box.min.x.floor() as i32;
                let max_x = self.bounding_box.max.x.ceil() as i32;
                let min_y = self.bounding_box.min.y.floor() as i32;
                let max_y = self.bounding_box.max.y.ceil() as i32;
                let min_z = self.bounding_box.min.z.floor() as i32;
                let max_z = self.bounding_box.max.z.ceil() as i32;

                for x in min_x..=max_x {
                    for z in min_z..=max_z {
                        for y in min_y..=max_y {
                            if let Some(block_type) = world.get_block_at(x, y, z) {
                                if block_type.is_solid() {
                                    if lowest_block_y_above.map_or(true, |lb| y < lb) {
                                        lowest_block_y_above = Some(y);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(block_y) = lowest_block_y_above {
                    self.position.y = block_y as f32 - HEIGHT - EPSILON;
                } else {
                    self.position.y = prev_position.y;
                }
            } else if self.velocity.y < 0.0 {
                self.position.y = prev_position.y;
                self.velocity.y = 0.0;
                self.on_ground = true;
            } else {
                self.position.y = prev_position.y;
            }
            self.update_bounding_box();
        } else {
            if self.velocity.y != 0.0 {
                self.on_ground = false;
            }
        }
        
        // --- 4. Handle horizontal movement with axis-separated resolution and step-up (Fixed step-up logic) ---
        let desired_x = desired_position.x;
        let desired_z = desired_position.z;
        let old_pos = self.position;
        let old_y = self.position.y;

        let can_step_up = self.velocity.y <= EPSILON;

        // --- X-axis movement resolution ---
        self.position.x = desired_x;
        self.update_bounding_box();

        if self.check_collision(world) {
            if can_step_up { // <-- Only try to step up if not moving upward
                // Collision in X: Try to step up
                self.position.y += STEP_HEIGHT;
                self.update_bounding_box();

                if self.check_collision(world) {
                    // Still colliding after step up: Revert X movement and reset Y
                    self.position.x = old_pos.x;
                    self.position.y = old_y;
                } else {
                    // Step-up succeeded! Y position is now elevated.
                }
            } else {
                // Cannot step up (e.g., jumping), so just revert horizontal movement.
                self.position.x = old_pos.x;
                self.position.y = old_y;
            }
        } else {
            self.position.y = old_y; // Restore Y if step-up was attempted and failed in the previous axis
        }
        
        self.update_bounding_box();

        // --- Z-axis movement resolution ---
        self.position.z = desired_z;
        self.update_bounding_box();
        
        if self.check_collision(world) {
            if can_step_up { // <-- Only try to step up if not moving upward
                // Collision in Z: Try to step up
                self.position.y = old_y + STEP_HEIGHT; // Always step up from the original Y for Z-check
                self.update_bounding_box();

                if self.check_collision(world) {
                    // Still colliding after step up: Revert Z movement and reset Y
                    self.position.z = old_pos.z;
                    self.position.y = old_y;
                } else {
                    // Step-up succeeded!
                }
            } else {
                // Cannot step up (e.g., jumping), so just revert horizontal movement.
                self.position.z = old_pos.z;
                self.position.y = old_y;
            }
        } else if self.position.y == old_y + STEP_HEIGHT {
            // If the X axis already lifted us, and Z didn't collide, keep the elevated Y.
        } else {
            self.position.y = old_y; // If no step-up occurred in Z, ensure Y is back to old_y
        }

        self.update_bounding_box();
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity.y = 8.0;
            self.on_ground = false;
        }
    }

    // check_collision remains unchanged as its logic is correct for an AABB check
    fn check_collision(&self, world: &World) -> bool {
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
    
    // can_fit is omitted for brevity as it is unused in physics, but should also use COLLISION_HALF_WIDTH
}