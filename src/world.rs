use crate::chunk::Chunk;
use crate::world_gen::WorldGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub seed: u32,
}

impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            seed,
        }
    }

    pub fn load_or_generate_chunk(&mut self, x: i32, z: i32, generator: &WorldGenerator) {
        self.chunks.entry((x, z)).or_insert_with(|| {
            generator.generate_chunk(x, z)
        });
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(self)?;
        fs::write(path, encoded)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if Path::new(path).exists() {
            let data = fs::read(path)?;
            let world = bincode::deserialize(&data)?;
            Ok(world)
        } else {
            Ok(World::new(12345))
        }
    }
}
