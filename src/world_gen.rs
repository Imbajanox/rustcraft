use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use crate::world::World;
use noise::{NoiseFn, Perlin};

// --- Neue Konstanten für erweiterte Weltgenerierung (FBM und Wasserlinie) ---

// Fractal Brownian Motion (FBM) Parameter für detaillierteres Terrain
const NUM_OCTAVES: u32 = 4;
const BASE_FREQUENCY: f64 = 0.008;
const PERSISTENCE: f64 = 0.5; // Steuert, wie stark Details in höheren Oktaven beitragen (Rauheit)
const LACUNARITY: f64 = 2.0; // Frequenzzunahme pro Oktave (Detaildichte)
const MIN_TREE_DISTANCE: i32 = 6; 

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

    

    pub fn should_generate_tree(&self, world_x: i32, world_z: i32) -> bool {
        if world_x % MIN_TREE_DISTANCE != 0 || world_z % MIN_TREE_DISTANCE != 0 {
            return false;
        }

        let height = self.get_height(world_x as f64, world_z as f64);
        let top_block_is_grass = height > WATER_LEVEL + 2; 
        let tree_noise = self.noise.get([world_x as f64 * 0.05, world_z as f64 * 0.05]);
        
        if top_block_is_grass && tree_noise > 0.6 {
            return true;
        }
        false
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
            
            }
        }

        chunk
    }

    pub fn place_trees(&self, world: &mut World, chunk_x: i32, chunk_z: i32) {
        // Wir iterieren über alle Blöcke DIESES Chunks, um mögliche Baumzentren zu finden.
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                
                let world_x = chunk_x * CHUNK_SIZE as i32 + x as i32;
                let world_z = chunk_z * CHUNK_SIZE as i32 + z as i32;
                
                // 1. DETERMINISTISCHE PRÜFUNG: Soll hier ein Baum wachsen?
                if self.should_generate_tree(world_x, world_z) {
                    
                    // --- KORREKTUR: Finde die tatsächliche Oberflächenhöhe im Chunk ---
                    let mut tree_height_y: usize = 0;
                    let mut found_surface = false;
                    
                    // Wir suchen von oben nach unten, um die y-Koordinate des Grasblocks zu finden.
                    // Wir verwenden world.get_chunk, um den gerade generierten Chunk abzufragen.
                    for y in (4..CHUNK_HEIGHT).rev() { 
                        // Wir fragen den Block in dem Chunk ab, für den wir die Features generieren.
                        let block = world.get_chunk(chunk_x, chunk_z)
                                         .map_or(BlockType::Air, |c| c.get_block(x, y, z));
                        
                        if block != BlockType::Air && block != BlockType::Water {
                            tree_height_y = y + 1; // Baum startet über diesem Block
                            found_surface = true;
                            break;
                        }
                    }

                    if !found_surface || tree_height_y < 5 {
                        continue; 
                    }
                    
                    // --- PLATZIERUNGSLOGIK ---
                    let trunk_height = 4;
                    let top_y = tree_height_y + trunk_height;
                    let top_y_i32 = top_y as i32;
                    let tree_height_y_i32 = tree_height_y as i32;

                    // 2. STAMM PLATZIEREN (Verwendet globale Koordinaten)
                    for y in tree_height_y_i32..top_y_i32 {
                        // set_block_at kümmert sich um die Chunk-Auflösung
                        world.set_block_at(world_x, y, world_z, BlockType::Wood);
                    }

                    // 3. BLÄTTER PLATZIEREN (Verwendet globale Koordinaten)
                    for dy in -2_i32..=1 { 
                        let leaf_y = top_y_i32 + dy; 
                        
                        // Hier setzen wir keinen y-Grenzwert, da set_block_at das tut
                        
                        let radius: i32 = match dy {
                            1 => 0, 0 => 1, _ => 2,
                        };

                        for dx in -radius..=radius {
                            for dz in -radius..=radius {
                                
                                // Eckblöcke im untersten Blattbereich entfernen
                                if radius == 2 && dx.abs() == 2 && dz.abs() == 2 { continue; }

                                let leaf_x = world_x + dx;
                                let leaf_z = world_z + dz;

                                // Stamm-Check
                                if dx == 0 && dz == 0 && leaf_y >= tree_height_y_i32 && leaf_y < top_y_i32 {
                                    continue; // Wir überspringen den Stamm
                                }
                                
                                // Nur setzen, wenn es Air, Leaves oder Water ist (oder ein anderer nicht-Stamm-Block)
                                if let Some(current_block) = world.get_block_at(leaf_x, leaf_y, leaf_z) {
                                    if current_block != BlockType::Wood {
                                        world.set_block_at(leaf_x, leaf_y, leaf_z, BlockType::Leaves);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}