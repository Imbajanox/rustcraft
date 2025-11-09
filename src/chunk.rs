use crate::block::BlockType;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pub blocks: Vec<BlockType>,
    pub x: i32,
    pub z: i32,
    #[serde(skip)]
    pub dirty: bool,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            blocks: vec![BlockType::Air; CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE],
            x,
            z,
            dirty: true,
        }
    }

    fn get_index(&self, x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        if x >= CHUNK_SIZE || y >= CHUNK_HEIGHT || z >= CHUNK_SIZE {
            return BlockType::Air;
        }
        self.blocks[self.get_index(x, y, z)]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        if x < CHUNK_SIZE && y < CHUNK_HEIGHT && z < CHUNK_SIZE {
            let index = self.get_index(x, y, z);
            self.blocks[index] = block;
            self.dirty = true;
        }
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}
