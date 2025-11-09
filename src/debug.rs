use crate::physics::Player;
use crate::raycast::raycast;
use crate::camera::Camera;
use crate::world::World;
use glam::Vec3;

pub struct DebugInfo {
    pub fps: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub looking_at_block: Option<(i32, i32, i32)>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self {
            fps: 0,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            on_ground: false,
            chunk_x: 0,
            chunk_z: 0,
            looking_at_block: None,
        }
    }

    pub fn update(&mut self, player: &Player, fps: u32, camera: &Camera, world: &World) {
        self.fps = fps;
        self.position = player.position;
        self.velocity = player.velocity;
        self.on_ground = player.on_ground;
        self.chunk_x = (player.position.x / 16.0).floor() as i32;
        self.chunk_z = (player.position.z / 16.0).floor() as i32;
        
        // Update looking at block with raycast
        let result = raycast(camera.position, camera.get_direction(), 10.0, world);
        self.looking_at_block = if result.hit {
            result.position
        } else {
            None
        };
    }

    pub fn format_display(&self) -> Vec<String> {
        vec![
            format!("=== DEBUG INFO (F3 to toggle) ==="),
            format!("FPS: {}", self.fps),
            format!("Position: ({:.2}, {:.2}, {:.2})", self.position.x, self.position.y, self.position.z),
            format!("Velocity: ({:.2}, {:.2}, {:.2})", self.velocity.x, self.velocity.y, self.velocity.z),
            format!("On Ground: {}", self.on_ground),
            format!("Chunk: ({}, {})", self.chunk_x, self.chunk_z),
            if let Some((x, y, z)) = self.looking_at_block {
                format!("Looking at: ({}, {}, {})", x, y, z)
            } else {
                "Looking at: None".to_string()
            },
        ]
    }
}
