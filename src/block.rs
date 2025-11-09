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

    pub fn get_texture_path(&self) -> Option<&'static str> {
        match self {
            BlockType::Air => None,
            BlockType::Dirt => Some("textures/dirt.png"),
            BlockType::Sand => Some("textures/sand.png"),
            BlockType::Grass => Some("textures/grass.png"),
            BlockType::Wood => Some("textures/wood.png"),
            BlockType::Leaves => Some("textures/leaves.png"),
            BlockType::Planks => Some("textures/planks.png"),
            BlockType::Glass => Some("textures/glass.png"),
            BlockType::Water => Some("textures/water.png"),
            BlockType::Stone => Some("textures/stone.png"),
        }
    }

    /// Return (col, row) coordinates of the block's tile inside the atlas.
    /// (0,0) is the left-bottom tile (adjust to your atlas orientation).
    /// Update these indices to match your atlas layout.
    pub fn atlas_coords(&self) -> Option<(u32, u32)> {
        match self {
            BlockType::Air => None,
            BlockType::Dirt => Some((0, 0)),
            BlockType::Sand => Some((1, 0)),
            BlockType::Grass => Some((2, 0)),
            BlockType::Wood => Some((3, 0)),
            BlockType::Leaves => Some((4, 0)),
            BlockType::Planks => Some((5, 0)),
            BlockType::Glass => Some((6, 0)),
            BlockType::Water => Some((7, 0)),
            BlockType::Stone => Some((8, 0)),
        }
    }
}
