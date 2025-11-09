use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use noise::{NoiseFn, Perlin};

// --- Neue Konstanten für erweiterte Weltgenerierung (FBM und Wasserlinie) ---

// Fractal Brownian Motion (FBM) Parameter für detaillierteres Terrain
const NUM_OCTAVES: u32 = 4;
const BASE_FREQUENCY: f64 = 0.008;
const PERSISTENCE: f64 = 0.5; // Steuert, wie stark Details in höheren Oktaven beitragen (Rauheit)
const LACUNARITY: f64 = 2.0; // Frequenzzunahme pro Oktave (Detaildichte)

// Allgemeine Parameter
pub const WATER_LEVEL: usize = 40; // Die Höhe der Meeresoberfläche

pub struct WorldGenerator {
    noise: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    // FBM (Fractal Brownian Motion) zur Generierung detaillierterer Höhe
    // Diese Funktion wird jetzt auch im main-Block verwendet, um die Spawn-Höhe zu bestimmen.
    pub fn get_height(&self, x: f64, z: f64) -> usize {
        let mut amplitude = 1.0;
        let mut frequency = BASE_FREQUENCY;
        let mut total_noise = 0.0;
        let mut total_amplitude = 0.0;
        
        // FBM implementieren, um das Rauschen über mehrere Oktaven zu mischen
        for _ in 0..NUM_OCTAVES {
            let noise_value = self.noise.get([x * frequency, z * frequency]);
            total_noise += noise_value * amplitude;
            total_amplitude += amplitude;
            
            amplitude *= PERSISTENCE;
            frequency *= LACUNARITY;
        }

        // Normalisieren (resultierendes Rauschen ist jetzt im Bereich [-1.0, 1.0])
        let normalized_noise = total_noise / total_amplitude; 
        
        // Skalieren und Verschieben zur gewünschten Höhe. 
        // Basis ist WATER_LEVEL + 15. Amplitude von 15.0 ergibt Höhen von ca. 40 bis 70.
        let height = (normalized_noise * 15.0 + (WATER_LEVEL as f64 + 15.0)) as usize; 
        
        // Sicherstellen, dass die Höhe innerhalb der Grenzen liegt
        height.min(CHUNK_HEIGHT - 5).max(1) 
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> Chunk {
        let mut chunk = Chunk::new(chunk_x, chunk_z);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = (chunk_x * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_z = (chunk_z * CHUNK_SIZE as i32 + z as i32) as f64;

                let height = self.get_height(world_x, world_z);
                let tree_noise = self.noise.get([world_x * 0.05, world_z * 0.05]);

                // --- Verbesserte Biome- und Schichtlogik ---
                
                // Bestimme die oberste feste Schicht (Strand vs. Grasland)
                let top_block = if height <= WATER_LEVEL + 2 {
                    BlockType::Sand // Niedriges Land wird Sand (Strand)
                } else {
                    BlockType::Grass // Höheres Land wird Grasland
                };
                
                // Bestimme die Subschicht (unter der obersten Schicht)
                let sub_block = if top_block == BlockType::Sand {
                    BlockType::Sand // Sand unter Sand (tieferer Sandstrand)
                } else {
                    BlockType::Dirt // Dirt unter Gras
                };

                for y in 0..CHUNK_HEIGHT {
                    let block = if y < height {
                        if y < height - 8 {
                            BlockType::Stone // Tiefste Schicht: Stein für Felsen
                        } else if y < height - 3 {
                            sub_block // Mittlere Schicht: Dirt oder Sand
                        } else {
                            top_block // Oberste Schicht: Gras oder Sand
                        }
                    } else if y < WATER_LEVEL {
                        BlockType::Water // Wasser bis zur Wasserlinie
                    } else {
                        BlockType::Air
                    };

                    chunk.set_block(x, y, z, block);
                }
                
                // Baum-Platzierungslogik: Nur auf Grasland platzieren
                if top_block == BlockType::Grass && tree_noise > 0.6 
                    && x > 2 && x < CHUNK_SIZE - 2 && z > 2 && z < CHUNK_SIZE - 2 {
                    // Baum-Stamm
                    chunk.set_block(x, height, z, BlockType::Wood);
                    chunk.set_block(x, height + 1, z, BlockType::Wood);
                    chunk.set_block(x, height + 2, z, BlockType::Wood);
                    chunk.set_block(x, height + 3, z, BlockType::Wood);
                    
                    // Blätter
                    for dx in -2_i32..=2 {
                        for dz in -2_i32..=2 {
                            for dy in 2..=4 {
                                if (dx.abs() + dz.abs()) < 4 && height + dy < CHUNK_HEIGHT {
                                    let leaf_x = (x as i32 + dx) as usize;
                                    let leaf_z = (z as i32 + dz) as usize;
                                    if leaf_x < CHUNK_SIZE && leaf_z < CHUNK_SIZE
                                        && chunk.get_block(leaf_x, height + dy, leaf_z) == BlockType::Air {
                                        chunk.set_block(leaf_x, height + dy, leaf_z, BlockType::Leaves);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        chunk
    }
}