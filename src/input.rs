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
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            left_mouse_pressed: false,
            right_mouse_pressed: false,
        }
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
        let sensitivity = 0.002;

        // Mouse look (no button hold required now)
        camera.yaw += self.mouse_delta.0 as f32 * sensitivity;
        camera.pitch -= self.mouse_delta.1 as f32 * sensitivity;
        camera.pitch = camera.pitch.clamp(-1.5, 1.5);

        self.mouse_delta = (0.0, 0.0);
    }

    pub fn update_player(&mut self, player: &mut Player, camera: &Camera, _delta_time: f32) {
        let speed = 4.3; // Walking speed in blocks/second

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
            movement = movement.normalize() * speed;
        }

        player.velocity.x = movement.x;
        player.velocity.z = movement.z;

        // Jumping
        if self.keys_pressed.contains(&KeyCode::Space) {
            player.jump();
        }
    }

    pub fn handle_block_interaction(&mut self, camera: &Camera, world: &mut World) -> bool {
        let mut world_changed = false;

        // Left click - destroy block
        if self.left_mouse_pressed {
            self.left_mouse_pressed = false; // Treat as single click
            let result = raycast(camera.position, camera.get_direction(), 5.0, world);
            if result.hit {
                if let Some((x, y, z)) = result.position {
                    world.set_block_at(x, y, z, BlockType::Air);
                    world_changed = true;
                }
            }
        }

        // Right click - place block
        if self.right_mouse_pressed {
            self.right_mouse_pressed = false; // Treat as single click
            let result = raycast(camera.position, camera.get_direction(), 5.0, world);
            if result.hit {
                if let (Some((x, y, z)), Some((nx, ny, nz))) = (result.position, result.normal) {
                    // Place block at the adjacent position
                    let place_x = x + nx;
                    let place_y = y + ny;
                    let place_z = z + nz;
                    world.set_block_at(place_x, place_y, place_z, BlockType::Dirt);
                    world_changed = true;
                }
            }
        }

        world_changed
    }
}
