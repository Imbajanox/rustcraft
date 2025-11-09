use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use noise::{NoiseFn, Perlin};

pub struct WorldGenerator {
    noise: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> Chunk {
        let mut chunk = Chunk::new(chunk_x, chunk_z);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = (chunk_x * CHUNK_SIZE as i32 + x as i32) as f64;
                let world_z = (chunk_z * CHUNK_SIZE as i32 + z as i32) as f64;

                // Generate base terrain height
                let height = self.get_height(world_x, world_z);
                
                // Generate different biome noise for vegetation
                let tree_noise = self.noise.get([world_x * 0.05, world_z * 0.05]);

                for y in 0..CHUNK_HEIGHT {
                    let block = if y < height {
                        if y < height - 4 {
                            BlockType::Dirt
                        } else if y < height - 1 {
                            // Check for sand near water level
                            if height < 35 {
                                BlockType::Sand
                            } else {
                                BlockType::Dirt
                            }
                        } else {
                            // Top layer
                            if height < 35 {
                                BlockType::Sand
                            } else {
                                BlockType::Grass
                            }
                        }
                    } else if y == height && height > 35 && tree_noise > 0.6 && x > 2 && x < CHUNK_SIZE - 2 && z > 2 && z < CHUNK_SIZE - 2 {
                        // Place tree trunk
                        chunk.set_block(x, y, z, BlockType::Wood);
                        chunk.set_block(x, y + 1, z, BlockType::Wood);
                        chunk.set_block(x, y + 2, z, BlockType::Wood);
                        chunk.set_block(x, y + 3, z, BlockType::Wood);
                        
                        // Place leaves
                        for dx in -2_i32..=2 {
                            for dz in -2_i32..=2 {
                                for dy in 2..=4 {
                                    if (dx.abs() + dz.abs()) < 4 && y + dy < CHUNK_HEIGHT {
                                        let leaf_x = (x as i32 + dx) as usize;
                                        let leaf_z = (z as i32 + dz) as usize;
                                        if leaf_x < CHUNK_SIZE && leaf_z < CHUNK_SIZE
                                            && chunk.get_block(leaf_x, y + dy, leaf_z) == BlockType::Air {
                                            chunk.set_block(leaf_x, y + dy, leaf_z, BlockType::Leaves);
                                        }
                                    }
                                }
                            }
                        }
                        BlockType::Air
                    } else {
                        BlockType::Air
                    };

                    chunk.set_block(x, y, z, block);
                }
            }
        }

        chunk
    }

    fn get_height(&self, x: f64, z: f64) -> usize {
        let scale1 = 0.01;
        let scale2 = 0.05;
        
        let noise1 = self.noise.get([x * scale1, z * scale1]);
        let noise2 = self.noise.get([x * scale2, z * scale2]);
        
        let combined = (noise1 * 0.7 + noise2 * 0.3) * 0.5 + 0.5;
        let height = (combined * 20.0 + 30.0) as usize;
        
        height.min(CHUNK_HEIGHT - 5)
    }
}
