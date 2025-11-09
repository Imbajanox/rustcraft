use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use crate::vertex::Vertex;
use crate::world::World;

pub struct MeshBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

const ATLAS_COLS: u32 = 9;      // number of tiles horizontally in atlas — set to your atlas layout
const ATLAS_ROWS: u32 = 1;      // number of tiles vertically in atlas
const TILE_PX: f32 = 16.0;

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    #[allow(clippy::too_many_arguments)]
    fn get_block_at(&self, world: &World, chunk: &Chunk, cx: usize, cy: usize, cz: usize, dx: i32, dy: i32, dz: i32) -> BlockType {
        let x = cx as i32 + dx;
        let y = cy as i32 + dy;
        let z = cz as i32 + dz;

        // Check if still within current chunk
        if x >= 0 && x < CHUNK_SIZE as i32 && y >= 0 && y < CHUNK_HEIGHT as i32 && z >= 0 && z < CHUNK_SIZE as i32 {
            return chunk.get_block(x as usize, y as usize, z as usize);
        }

        // Check if out of world height bounds
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }

        // Calculate which chunk the neighbor is in
        let world_x = chunk.x * CHUNK_SIZE as i32 + x;
        let world_z = chunk.z * CHUNK_SIZE as i32 + z;
        let neighbor_chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let neighbor_chunk_z = world_z.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;

        // Get block from neighbor chunk
        if let Some(neighbor_chunk) = world.get_chunk(neighbor_chunk_x, neighbor_chunk_z) {
            neighbor_chunk.get_block(local_x, y as usize, local_z)
        } else {
            BlockType::Air
        }
    }

    pub fn build_chunk_mesh(&mut self, chunk: &Chunk, world: &World) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block = chunk.get_block(x, y, z);
                    if block.is_solid() {
                        let world_x = (chunk.x * CHUNK_SIZE as i32 + x as i32) as f32;
                        let world_y = y as f32;
                        let world_z = (chunk.z * CHUNK_SIZE as i32 + z as i32) as f32;

                        self.add_block_faces(
                            world_x,
                            world_y,
                            world_z,
                            block,
                            chunk,
                            world,
                            x,
                            y,
                            z,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_block_faces(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        block: BlockType,
        chunk: &Chunk,
        world: &World,
        cx: usize,
        cy: usize,
        cz: usize,
    ) {
        let color = block.get_color();
        let tile = block.atlas_coords().unwrap_or((0, 0));

        // Top face
        if self.get_block_at(world, chunk, cx, cy, cz, 0, 1, 0).is_transparent() {
            self.add_face(
                x,
                y + 1.0,
                z,
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                color,
                1.0,
                tile,
            );
        }

        // Bottom face
        if self.get_block_at(world, chunk, cx, cy, cz, 0, -1, 0).is_transparent() {
            self.add_face(
                x,
                y,
                z,
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                color,
                0.5,
                tile,
            );
        }

        // Front face (+Z)
        if self.get_block_at(world, chunk, cx, cy, cz, 0, 0, 1).is_transparent() {
            self.add_face(
                x,
                y,
                z + 1.0,
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                color,
                0.8,
                tile,
            );
        }

        // Back face (-Z)
        if self.get_block_at(world, chunk, cx, cy, cz, 0, 0, -1).is_transparent() {
            self.add_face(
                x,
                y,
                z,
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                color,
                0.8,
                tile,
            );
        }

        // Right face (+X)
        if self.get_block_at(world, chunk, cx, cy, cz, 1, 0, 0).is_transparent() {
            self.add_face(
                x + 1.0,
                y,
                z + 1.0,    
                [0.0, 0.0, -1.0], // changed to point u so u x v = +X (outward)
                [0.0, 1.0, 0.0],
                color,
                0.7,
                tile,
            );
        }

        // Left face (-X)
        if self.get_block_at(world, chunk, cx, cy, cz, -1, 0, 0).is_transparent() {
            self.add_face(
                x,
                y,
                z,
                [0.0, 0.0, 1.0], // changed so u x v = -X (outward for left face)
                [0.0, 1.0, 0.0],
                color,
                0.7,
                tile,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_face(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        u: [f32; 3],
        v: [f32; 3],
        base_color: [f32; 3],
        shade: f32,
        tile: (u32, u32),
    ) {
        let color = [
            base_color[0] * shade,
            base_color[1] * shade,
            base_color[2] * shade,
        ];

        // compute UV rectangle for this tile
        let tile_w = 1.0 / ATLAS_COLS as f32;
        let tile_h = 1.0 / ATLAS_ROWS as f32;

        // small inset in UV space to avoid bleeding from neighbors (0.5 texel)
        let inset_u = 0.5 / (ATLAS_COLS as f32 * TILE_PX);
        let inset_v = 0.5 / (ATLAS_ROWS as f32 * TILE_PX);

        let u0 = tile.0 as f32 * tile_w + inset_u;
        let v0 = tile.1 as f32 * tile_h + inset_v;
        let u1 = u0 + tile_w - 2.0 * inset_u;
        let v1 = v0 + tile_h - 2.0 * inset_v;

        let base_idx = self.vertices.len() as u32;

        // Define UV coordinates for a face (bottom-left, bottom-right, top-right, top-left)
        self.vertices.push(Vertex {
            position: [x, y, z],
            color,
            tex_coords: [u0, v0],
        });
        self.vertices.push(Vertex {
            position: [x + u[0], y + u[1], z + u[2]],
            color,
            tex_coords: [u1, v0],
        });
        self.vertices.push(Vertex {
            position: [x + u[0] + v[0], y + u[1] + v[1], z + u[2] + v[2]],
            color,
            tex_coords: [u1, v1],
        });
        self.vertices.push(Vertex {
            position: [x + v[0], y + v[1], z + v[2]],
            color,
            tex_coords: [u0, v1],
        });

        // Two triangles per face
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx + 3);
    }
}
