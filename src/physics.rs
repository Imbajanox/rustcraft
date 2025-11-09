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
        const HALF_WIDTH: f32 = 0.3;
        const HEIGHT: f32 = 1.8;

        // Keep previous position so we can reason about where we came from
        let prev_position = self.position;
        let prev_feet_y = prev_position.y;

        // --- NEW: Check support under feet when currently on_ground ---
        // If the block(s) under the player's feet were removed since last frame, we must clear on_ground
        // so gravity starts to apply immediately (previously we only cleared on_ground when velocity.y != 0).
        if self.on_ground {
            // small probe distance under feet to avoid jitter from tiny EPSILON differences
            let support_probe = 0.05_f32;

            // x/z block range covered by the player's footprint
            let min_x = self.bounding_box.min.x.floor() as i32;
            let max_x = self.bounding_box.max.x.ceil() as i32;
            let min_z = self.bounding_box.min.z.floor() as i32;
            let max_z = self.bounding_box.max.z.ceil() as i32;

            let feet_y = self.position.y;
            let mut supported = false;

            // Candidate block y values roughly under the feet (floor(feet_y)-1 .. floor(feet_y))
            let y0 = (feet_y.floor() as i32) - 1;
            let y1 = feet_y.floor() as i32 + 1;

            'support_y: for y in y0..=y1 {
                let block_top = y as f32 + 1.0;
                // Only consider block tops reasonably close under feet
                if block_top >= feet_y - support_probe && block_top <= feet_y + support_probe {
                    for x in min_x..=max_x {
                        for z in min_z..=max_z {
                            if let Some(block_type) = world.get_block_at(x, y, z) {
                                if block_type.is_solid() {
                                    supported = true;
                                    break 'support_y;
                                }
                            }
                        }
                    }
                }
            }

            if !supported {
                // Lost support -> start falling
                self.on_ground = false;
                // leave velocity.y as-is (0.0). Gravity is applied below because now !on_ground.
            }
        }

        // Apply gravity
        if !self.on_ground {
            self.velocity.y += GRAVITY * delta_time;
            if self.velocity.y < TERMINAL_VELOCITY {
                self.velocity.y = TERMINAL_VELOCITY;
            }
        }

        // Calculate target position (full step)
        let desired_position = self.position + self.velocity * delta_time;

        // --- VERTICAL SWEEP to avoid tunneling ---
        // If we're moving down, check integer block layers between prev_feet_y and desired_feet_y
        if self.velocity.y < 0.0 {
            let desired_feet_y = desired_position.y;

            // Build swept X/Z ranges using union of previous and desired AABBs (covers horizontal motion during fall)
            let prev_bb = Aabb::from_position(prev_position, HALF_WIDTH, HEIGHT);
            let mut temp_pos = prev_position;
            temp_pos.y = desired_position.y;
            let desired_bb = Aabb::from_position(temp_pos, HALF_WIDTH, HEIGHT);

            let swept_min_x = prev_bb.min.x.min(desired_bb.min.x).floor() as i32;
            let swept_max_x = prev_bb.max.x.max(desired_bb.max.x).ceil() as i32;
            let swept_min_z = prev_bb.min.z.min(desired_bb.min.z).floor() as i32;
            let swept_max_z = prev_bb.max.z.max(desired_bb.max.z).ceil() as i32;

            // integer y range to check (from floor(desired_feet_y) up to floor(prev_feet_y))
            let check_min_y = desired_feet_y.floor() as i32 - 1; // small margin
            let check_max_y = prev_feet_y.floor() as i32 + 1;

            let mut landed = false;
            let mut landing_y: Option<i32> = None;

            for y in check_min_y..=check_max_y {
                // block top coordinate
                let block_top = y as f32 + 1.0;
                // only consider tops that lie between desired_feet_y and prev_feet_y (with small epsilon)
                if block_top > prev_feet_y + EPSILON || block_top < desired_feet_y - EPSILON {
                    continue;
                }

                'xz_loop: for x in swept_min_x..=swept_max_x {
                    for z in swept_min_z..=swept_max_z {
                        if let Some(block_type) = world.get_block_at(x, y, z) {
                            if block_type.is_solid() {
                                // Found a block we cross - snap to its top
                                landing_y = Some(y);
                                landed = true;
                                break 'xz_loop;
                            }
                        }
                    }
                }

                if landed {
                    break;
                }
            }

            if let Some(by) = landing_y {
                // Snap feet on top of block, zero vertical velocity and set on_ground
                self.position.y = by as f32 + 1.0 + EPSILON;
                self.velocity.y = 0.0;
                self.on_ground = true;
                self.update_bounding_box();
            } else {
                // No block encountered on the vertical path -> proceed to set desired Y (we might still collide sideways below)
                self.position.y = desired_position.y;
                self.update_bounding_box();
            }
        } else {
            // Not falling (rising or stationary): apply desired Y and handle collision as before
            self.position.y = desired_position.y;
            self.update_bounding_box();
        }

        // If after the sweep we still intersect something vertically (rare), run the previous collision resolution
        if self.check_collision(world) {
            if self.velocity.y < 0.0 {
                // Falling - fallback landing logic (only accept blocks whose top <= previous feet)
                self.velocity.y = 0.0;

                let min_x = self.bounding_box.min.x.floor() as i32;
                let max_x = self.bounding_box.max.x.ceil() as i32;
                let min_y = (self.bounding_box.min.y.floor() as i32) - 1; // small margin
                let max_y = self.bounding_box.max.y.ceil() as i32;
                let min_z = self.bounding_box.min.z.floor() as i32;
                let max_z = self.bounding_box.max.z.ceil() as i32;

                let mut highest_block_y: Option<i32> = None;

                for x in min_x..=max_x {
                    for z in min_z..=max_z {
                        for y in min_y..=max_y {
                            if let Some(block_type) = world.get_block_at(x, y, z) {
                                if block_type.is_solid() {
                                    let block_top = y as f32 + 1.0;
                                    if block_top <= prev_feet_y + EPSILON {
                                        if highest_block_y.map_or(true, |hb| y > hb) {
                                            highest_block_y = Some(y);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(block_y) = highest_block_y {
                    self.position.y = block_y as f32 + 1.0 + EPSILON;
                    self.on_ground = true;
                } else {
                    self.position.y = prev_position.y;
                }
            } else {
                // Rising - ceiling
                self.velocity.y = 0.0;

                let min_x = self.bounding_box.min.x.floor() as i32;
                let max_x = self.bounding_box.max.x.ceil() as i32;
                let min_y = self.bounding_box.min.y.floor() as i32;
                let max_y = self.bounding_box.max.y.ceil() as i32;
                let min_z = self.bounding_box.min.z.floor() as i32;
                let max_z = self.bounding_box.max.z.ceil() as i32;

                let mut lowest_block_y_above: Option<i32> = None;

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
            }

            self.update_bounding_box();
        } else {
            // If we didn't land in the sweep, ensure on_ground false
            if self.velocity.y != 0.0 {
                self.on_ground = false;
            }
        }

        // --- Handle horizontal movement with axis-separated resolution (unchanged) ---
        let old_x = self.position.x;
        let old_z = self.position.z;

        // Move in X
        self.position.x = desired_position.x;
        self.update_bounding_box();

        if self.check_collision(world) {
            // Collect blocking blocks overlapping in Y and Z to compute minimal X correction
            let mut nearest_block_min_x: Option<f32> = None;
            let mut nearest_block_max_x: Option<f32> = None;

            let min_x = self.bounding_box.min.x.floor() as i32 - 1;
            let max_x = self.bounding_box.max.x.ceil() as i32 + 1;
            let min_y = self.bounding_box.min.y.floor() as i32;
            let max_y = self.bounding_box.max.y.ceil() as i32;
            let min_z = self.bounding_box.min.z.floor() as i32 - 1;
            let max_z = self.bounding_box.max.z.ceil() as i32 + 1;

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    for z in min_z..=max_z {
                        if let Some(block_type) = world.get_block_at(x, y, z) {
                            if block_type.is_solid() {
                                let block_aabb = Aabb::new(
                                    Vec3::new(x as f32, y as f32, z as f32),
                                    Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                                );

                                // Check overlap in Y and Z (we're resolving X)
                                if self.bounding_box.min.y < block_aabb.max.y
                                    && self.bounding_box.max.y > block_aabb.min.y
                                    && self.bounding_box.min.z < block_aabb.max.z
                                    && self.bounding_box.max.z > block_aabb.min.z
                                {
                                    nearest_block_min_x = Some(nearest_block_min_x.map_or(block_aabb.min.x, |v| v.min(block_aabb.min.x)));
                                    nearest_block_max_x = Some(nearest_block_max_x.map_or(block_aabb.max.x, |v| v.max(block_aabb.max.x)));
                                }
                            }
                        }
                    }
                }
            }

            if self.velocity.x > 0.0 {
                if let Some(block_min_x) = nearest_block_min_x {
                    self.position.x = block_min_x - HALF_WIDTH - EPSILON;
                } else {
                    self.position.x = old_x;
                }
            } else if self.velocity.x < 0.0 {
                if let Some(block_max_x) = nearest_block_max_x {
                    self.position.x = block_max_x + HALF_WIDTH + EPSILON;
                } else {
                    self.position.x = old_x;
                }
            } else {
                self.position.x = old_x;
            }

            self.update_bounding_box();
        }

        // Move in Z
        self.position.z = desired_position.z;
        self.update_bounding_box();

        if self.check_collision(world) {
            // Collect blocking blocks overlapping in X and Y to compute minimal Z correction
            let mut nearest_block_min_z: Option<f32> = None;
            let mut nearest_block_max_z: Option<f32> = None;

            let min_x = self.bounding_box.min.x.floor() as i32 - 1;
            let max_x = self.bounding_box.max.x.ceil() as i32 + 1;
            let min_y = self.bounding_box.min.y.floor() as i32;
            let max_y = self.bounding_box.max.y.ceil() as i32;
            let min_z = self.bounding_box.min.z.floor() as i32 - 1;
            let max_z = self.bounding_box.max.z.ceil() as i32 + 1;

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    for z in min_z..=max_z {
                        if let Some(block_type) = world.get_block_at(x, y, z) {
                            if block_type.is_solid() {
                                let block_aabb = Aabb::new(
                                    Vec3::new(x as f32, y as f32, z as f32),
                                    Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32),
                                );

                                // Check overlap in X and Y (we're resolving Z)
                                if self.bounding_box.min.y < block_aabb.max.y
                                    && self.bounding_box.max.y > block_aabb.min.y
                                    && self.bounding_box.min.x < block_aabb.max.x
                                    && self.bounding_box.max.x > block_aabb.min.x
                                {
                                    nearest_block_min_z = Some(nearest_block_min_z.map_or(block_aabb.min.z, |v| v.min(block_aabb.min.z)));
                                    nearest_block_max_z = Some(nearest_block_max_z.map_or(block_aabb.max.z, |v| v.max(block_aabb.max.z)));
                                }
                            }
                        }
                    }
                }
            }

            if self.velocity.z > 0.0 {
                if let Some(block_min_z) = nearest_block_min_z {
                    self.position.z = block_min_z - HALF_WIDTH - EPSILON;
                } else {
                    self.position.z = old_z;
                }
            } else if self.velocity.z < 0.0 {
                if let Some(block_max_z) = nearest_block_max_z {
                    self.position.z = block_max_z + HALF_WIDTH + EPSILON;
                } else {
                    self.position.z = old_z;
                }
            } else {
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
