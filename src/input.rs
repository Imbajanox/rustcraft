use crate::camera::Camera;
use crate::physics::Player;
use crate::raycast::raycast;
use crate::world::World;
use crate::block::BlockType;
use std::collections::HashSet;
use winit::event::*;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputHandler {
    keys_pressed: HashSet<KeyCode>,
    pub mouse_delta: (f64, f64),
    pub left_mouse_pressed: bool,
    pub right_mouse_pressed: bool,
    sensitivity: f32,
    walk_speed: f32,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            left_mouse_pressed: false,
            right_mouse_pressed: false,
            sensitivity: 0.005,
            walk_speed: 4.3,
        }
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    pub fn set_walk_speed(&mut self, speed: f32) {
        self.walk_speed = speed;
    }

    pub fn process_keyboard(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.keys_pressed.insert(keycode);
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&keycode);
                }
            }
        }
    }

    pub fn process_mouse_motion(&mut self, delta: (f64, f64)) {
        self.mouse_delta = delta;
    }

    pub fn process_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        match button {
            MouseButton::Left => {
                self.left_mouse_pressed = state == ElementState::Pressed;
            }
            MouseButton::Right => {
                self.right_mouse_pressed = state == ElementState::Pressed;
            }
            _ => {}
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        // Mouse look (no button hold required now)
        camera.yaw += self.mouse_delta.0 as f32 * self.sensitivity;
        camera.pitch -= self.mouse_delta.1 as f32 * self.sensitivity;
        camera.pitch = camera.pitch.clamp(-1.5, 1.5);

        self.mouse_delta = (0.0, 0.0);
    }

    pub fn update_player(&mut self, player: &mut Player, camera: &Camera, _delta_time: f32) {
        let mut movement = glam::Vec3::ZERO;

        // Horizontal movement
        if self.keys_pressed.contains(&KeyCode::KeyW) {
            movement += camera.get_forward();
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            movement -= camera.get_forward();
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            movement -= camera.get_right();
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            movement += camera.get_right();
        }

        // Normalize horizontal movement to prevent faster diagonal movement
        if movement.length_squared() > 0.0 {
            movement = movement.normalize() * self.walk_speed;
        }

        player.velocity.x = movement.x;
        player.velocity.z = movement.z;

        // Jumping
        if self.keys_pressed.contains(&KeyCode::Space) {
            player.jump();
        }
    }

    pub fn handle_block_interaction(&mut self, camera: &Camera, world: &mut World, _ui: &crate::ui::UiRenderer, player_pos: glam::Vec3) -> (bool, bool) {
        let mut world_changed = false;
        let mut removed_under_feet = false;

        // Left click - destroy block and add to inventory
        if self.left_mouse_pressed {
            self.left_mouse_pressed = false; // Treat as single click
            let result = raycast(camera.position, camera.get_direction(), 5.0, world);
            if result.hit {
                if let Some((x, y, z)) = result.position {
                    // Get the block type before destroying it
                    if let Some(block_type) = world.get_block_at(x, y, z) {
                        if block_type != BlockType::Air {
                            let success = world.set_block_at(x, y, z, BlockType::Air);
                            if success {
                                // Add destroyed block to inventory
                                world.inventory.add_item(block_type, 1);
                                world_changed = true;

                                // Check whether the removed block was directly under the player's feet.
                                // Player's feet world coordinate is player_pos.y, block occupies [y, y+1).
                                let foot_block_x = player_pos.x.floor() as i32;
                                let foot_block_z = player_pos.z.floor() as i32;
                                let feet_floor_y = player_pos.y.floor() as i32;
                                // block is directly under feet if it's the block whose top is at feet_floor_y
                                // i.e. block y == feet_floor_y - 1 and x/z cell matches footprint.
                                if x == foot_block_x && z == foot_block_z && y == feet_floor_y - 1 {
                                    removed_under_feet = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Right click - place block from inventory
        if self.right_mouse_pressed {
            self.right_mouse_pressed = false; // Treat as single click
            
            // Check if player has the selected block in inventory
            if world.inventory.has_selected_item() {
                let result = raycast(camera.position, camera.get_direction(), 5.0, world);
                if result.hit {
                    if let (Some((x, y, z)), Some((nx, ny, nz))) = (result.position, result.normal) {
                        // Place block at the adjacent position
                        let place_x = x + nx;
                        let place_y = y + ny;
                        let place_z = z + nz;
                        
                        // Get the block type from inventory
                        if let Some(block_type) = world.inventory.get_selected_block() {
                            if world.set_block_at(place_x, place_y, place_z, block_type) {
                                // Remove one block from inventory
                                world.inventory.remove_selected_item(1);
                                world_changed = true;
                            }
                        }
                    }
                }
            }
        }

        (world_changed, removed_under_feet)
    }
}
