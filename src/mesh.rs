use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_HEIGHT, CHUNK_SIZE};
use crate::vertex::Vertex;

pub struct MeshBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn build_chunk_mesh(&mut self, chunk: &Chunk) {
        self.vertices.clear();
        self.indices.clear();

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
        cx: usize,
        cy: usize,
        cz: usize,
    ) {
        let color = block.get_color();

        // Top face
        if cy + 1 >= CHUNK_HEIGHT || chunk.get_block(cx, cy + 1, cz).is_transparent() {
            self.add_face(
                x,
                y + 1.0,
                z,
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                color,
                1.0,
            );
        }

        // Bottom face
        if cy == 0 || chunk.get_block(cx, cy - 1, cz).is_transparent() {
            self.add_face(
                x,
                y,
                z,
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                color,
                0.5,
            );
        }

        // Front face (+Z)
        if cz + 1 >= CHUNK_SIZE || chunk.get_block(cx, cy, cz + 1).is_transparent() {
            self.add_face(
                x,
                y,
                z + 1.0,
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                color,
                0.8,
            );
        }

        // Back face (-Z)
        if cz == 0 || chunk.get_block(cx, cy, cz - 1).is_transparent() {
            self.add_face(
                x,
                y,
                z,
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                color,
                0.8,
            );
        }

        // Right face (+X)
        if cx + 1 >= CHUNK_SIZE || chunk.get_block(cx + 1, cy, cz).is_transparent() {
            self.add_face(
                x + 1.0,
                y,
                z,
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0],
                color,
                0.7,
            );
        }

        // Left face (-X)
        if cx == 0 || chunk.get_block(cx - 1, cy, cz).is_transparent() {
            self.add_face(
                x,
                y,
                z,
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0],
                color,
                0.7,
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
    ) {
        let color = [
            base_color[0] * shade,
            base_color[1] * shade,
            base_color[2] * shade,
        ];

        let base_idx = self.vertices.len() as u32;

        self.vertices.push(Vertex {
            position: [x, y, z],
            color,
        });
        self.vertices.push(Vertex {
            position: [x + u[0], y + u[1], z + u[2]],
            color,
        });
        self.vertices.push(Vertex {
            position: [x + u[0] + v[0], y + u[1] + v[1], z + u[2] + v[2]],
            color,
        });
        self.vertices.push(Vertex {
            position: [x + v[0], y + v[1], z + v[2]],
            color,
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
