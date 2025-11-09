use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub sensitivity: f32,
    pub walk_speed: f32,
    pub view_distance: i32,
    pub fov: f32,
    pub show_debug: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.005,
            walk_speed: 4.3,
            view_distance: 6,
            fov: 70.0,
            show_debug: false,
        }
    }
}

impl GameConfig {
    pub fn load(path: &str) -> Self {
        if Path::new(path).exists() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&data) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }
}
