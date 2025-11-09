use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Air,
    Dirt,
    Sand,
    Grass,
    Wood,
    Leaves,
    Planks,
    Glass,
    Water,
    Stone,
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air)
    }

    pub fn is_transparent(&self) -> bool {
        matches!(self, BlockType::Air | BlockType::Glass | BlockType::Leaves | BlockType::Water)
    }

    pub fn get_color(&self) -> [f32; 3] {
        match self {
            BlockType::Air => [0.0, 0.0, 0.0],
            BlockType::Dirt => [0.55, 0.27, 0.07],
            BlockType::Sand => [0.76, 0.70, 0.50],
            BlockType::Grass => [0.13, 0.55, 0.13],
            BlockType::Wood => [0.40, 0.26, 0.13],
            BlockType::Leaves => [0.0, 0.39, 0.0],
            BlockType::Planks => [0.72, 0.52, 0.04],
            BlockType::Glass => [0.8, 0.9, 1.0],
            BlockType::Water => [0.0, 0.4, 0.8],
            BlockType::Stone => [0.5, 0.5, 0.5],
        }
    }
}
