use crate::camera::Camera;
use std::collections::HashSet;
use winit::event::*;
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputHandler {
    keys_pressed: HashSet<KeyCode>,
    pub mouse_delta: (f64, f64),
    pub mouse_pressed: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_pressed: false,
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
        if button == MouseButton::Left {
            self.mouse_pressed = state == ElementState::Pressed;
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
        let speed = 10.0 * delta_time;
        let sensitivity = 0.002;

        // Mouse look
        if self.mouse_pressed {
            camera.yaw += self.mouse_delta.0 as f32 * sensitivity;
            camera.pitch -= self.mouse_delta.1 as f32 * sensitivity;
            camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        }
        self.mouse_delta = (0.0, 0.0);

        // Movement
        if self.keys_pressed.contains(&KeyCode::KeyW) {
            camera.position += camera.get_forward() * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            camera.position -= camera.get_forward() * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            camera.position -= camera.get_right() * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            camera.position += camera.get_right() * speed;
        }
        if self.keys_pressed.contains(&KeyCode::Space) {
            camera.position.y += speed;
        }
        if self.keys_pressed.contains(&KeyCode::ShiftLeft) {
            camera.position.y -= speed;
        }
    }
}
