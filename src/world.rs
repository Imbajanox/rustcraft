use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_SIZE, CHUNK_HEIGHT};
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
        use std::collections::hash_map::Entry;

        let is_newly_generated = match self.chunks.entry((x, z)) {
            Entry::Occupied(_) => {
                // Der Chunk existiert bereits, nichts zu tun.
                false
            }, 
            Entry::Vacant(entry) => {
                // 1. Chunk generieren und Terrain/Blöcke füllen (OHNE Bäume!)
                let new_chunk = generator.generate_chunk(x, z);
                
                // Chunk in die HashMap einfügen
                entry.insert(new_chunk);

                // Chunk wurde neu generiert
                true 
            }
        };

        // --- GLOBALER FEATURE-PLATZIERUNGS-SCHRITT ---
        // Dies muss außerhalb des 'match' erfolgen, damit wir das gesamte World-Objekt 
        // als mutable Referenz (self) verwenden können, um Blöcke global zu setzen.
        if is_newly_generated {
            // Bäume global platzieren, was die set_block_at Methode der World verwendet
            // Die Bäume werden nun über Chunk-Grenzen hinweg in benachbarten Chunks gesetzt.
            generator.place_trees(self, x, z);
            
            // --- Logik: Nachbarn als Dirty markieren ---
            // Markiere alle 9 Chunks (den aktuellen und 8 Nachbarn) als 'dirty', da Bäume 
            // sowohl in den aktuellen Chunk als auch in die Nachbarn hineinragen können.
            for dx in -1..=1 {
                for dz in -1..=1 {
                    if let Some(neighbor_chunk) = self.chunks.get_mut(&(x + dx, z + dz)) {
                        // Den Nachbarn markieren, um sein Mesh zu aktualisieren.
                        neighbor_chunk.mark_dirty(); 
                    }
                }
            }
        }
    }


    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, z))
    }

    pub fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(x, z))
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Option<BlockType> {
        // Check if y is within valid bounds
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return Some(BlockType::Air);
        }

        // Calculate which chunk this block belongs to
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = z.div_euclid(CHUNK_SIZE as i32);
        
        // Calculate local coordinates within the chunk
        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

        // Get the chunk and the block
        self.get_chunk(chunk_x, chunk_z)
            .map(|chunk| chunk.get_block(local_x, y as usize, local_z))
    }

    pub fn set_block_at(&mut self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        // Check if y is within valid bounds
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return false;
        }

        // Calculate which chunk this block belongs to
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = z.div_euclid(CHUNK_SIZE as i32);
        
        // Calculate local coordinates within the chunk
        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

        // Set the block
        if let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_z) {
            chunk.set_block(local_x, y as usize, local_z, block);
            
            // Mark neighboring chunks as dirty if block is on chunk edge
            if local_x == 0 {
                if let Some(neighbor) = self.get_chunk_mut(chunk_x - 1, chunk_z) {
                    neighbor.mark_dirty();
                }
            } else if local_x == CHUNK_SIZE - 1 {
                if let Some(neighbor) = self.get_chunk_mut(chunk_x + 1, chunk_z) {
                    neighbor.mark_dirty();
                }
            }
            
            if local_z == 0 {
                if let Some(neighbor) = self.get_chunk_mut(chunk_x, chunk_z - 1) {
                    neighbor.mark_dirty();
                }
            } else if local_z == CHUNK_SIZE - 1 {
                if let Some(neighbor) = self.get_chunk_mut(chunk_x, chunk_z + 1) {
                    neighbor.mark_dirty();
                }
            }
            
            true
        } else {
            false
        }
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