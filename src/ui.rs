use bytemuck::{Pod, Zeroable};
use crate::block::BlockType;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct UiVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl UiVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct UiRenderer {
    pub selected_block: BlockType,
    crosshair_vertices: Vec<UiVertex>,
    crosshair_indices: Vec<u32>,
    toolbar_vertices: Vec<UiVertex>,
    toolbar_indices: Vec<u32>,
}

impl UiRenderer {
    pub fn new() -> Self {
        let mut ui = Self {
            selected_block: BlockType::Dirt,
            crosshair_vertices: Vec::new(),
            crosshair_indices: Vec::new(),
            toolbar_vertices: Vec::new(),
            toolbar_indices: Vec::new(),
        };
        ui.build_crosshair();
        ui.build_toolbar();
        ui
    }

    fn build_crosshair(&mut self) {
        self.crosshair_vertices.clear();
        self.crosshair_indices.clear();

        let size = 0.015; // Size of crosshair in NDC
        let thickness = 0.003;
        let gap = 0.01;
        let white = [1.0, 1.0, 1.0, 1.0];

        // Horizontal line (left part)
        let left_start = -(gap + size);
        let left_end = -gap;
        self.add_line(left_start, 0.0, left_end, 0.0, thickness, white);

        // Horizontal line (right part)
        let right_start = gap;
        let right_end = gap + size;
        self.add_line(right_start, 0.0, right_end, 0.0, thickness, white);

        // Vertical line (top part)
        let top_start = gap;
        let top_end = gap + size;
        self.add_line(0.0, top_start, 0.0, top_end, thickness, white);

        // Vertical line (bottom part)
        let bottom_start = -(gap + size);
        let bottom_end = -gap;
        self.add_line(0.0, bottom_start, 0.0, bottom_end, thickness, white);
    }

    pub fn build_toolbar(&mut self) {
        self.toolbar_vertices.clear();
        self.toolbar_indices.clear();

        let toolbar_width = 0.6;
        let toolbar_height = 0.08;
        let y_pos = -0.9; // Bottom of screen
        let num_slots = 9;
        let slot_size = toolbar_width / num_slots as f32;
        let border_thickness = 0.004;

        // Draw toolbar background
        let bg_color = [0.0, 0.0, 0.0, 0.5];
        self.add_rect(
            -toolbar_width / 2.0,
            y_pos,
            toolbar_width,
            toolbar_height,
            bg_color,
        );

        // Draw slot borders
        for i in 0..num_slots {
            let x = -toolbar_width / 2.0 + i as f32 * slot_size;
            let border_color = [0.8, 0.8, 0.8, 0.8];
            
            // Draw border as outline
            self.add_rect_outline(x, y_pos, slot_size, toolbar_height, border_thickness, border_color);
        }

        // Highlight selected slot based on current selected_block
        let block_types = [
            BlockType::Dirt,
            BlockType::Grass,
            BlockType::Sand,
            BlockType::Wood,
            BlockType::Planks,
            BlockType::Leaves,
            BlockType::Glass,
            BlockType::Water,
        ];
        
        let selected_slot = block_types.iter().position(|&b| b == self.selected_block).unwrap_or(0);
        let x = -toolbar_width / 2.0 + selected_slot as f32 * slot_size;
        let highlight_color = [1.0, 1.0, 1.0, 1.0];
        self.add_rect_outline(x, y_pos, slot_size, toolbar_height, border_thickness * 2.0, highlight_color);

        // Draw colored blocks in slots to represent block types
        for (i, block_type) in block_types.iter().enumerate() {
            let x = -toolbar_width / 2.0 + i as f32 * slot_size;
            let padding = slot_size * 0.2;
            let block_size = slot_size - 2.0 * padding;
            let color = block_type.get_color();
            let block_color = [color[0], color[1], color[2], 1.0];
            
            self.add_rect(
                x + padding,
                y_pos + padding,
                block_size,
                toolbar_height - 2.0 * padding,
                block_color,
            );
        }
    }

    fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: [f32; 4]) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        
        if len < 0.0001 {
            return;
        }

        // Perpendicular vector
        let px = -dy / len * thickness;
        let py = dx / len * thickness;

        let base_idx = self.crosshair_vertices.len() as u32;

        self.crosshair_vertices.push(UiVertex {
            position: [x1 - px, y1 - py],
            color,
        });
        self.crosshair_vertices.push(UiVertex {
            position: [x1 + px, y1 + py],
            color,
        });
        self.crosshair_vertices.push(UiVertex {
            position: [x2 + px, y2 + py],
            color,
        });
        self.crosshair_vertices.push(UiVertex {
            position: [x2 - px, y2 - py],
            color,
        });

        self.crosshair_indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    }

    fn add_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        let base_idx = self.toolbar_vertices.len() as u32;

        self.toolbar_vertices.push(UiVertex {
            position: [x, y],
            color,
        });
        self.toolbar_vertices.push(UiVertex {
            position: [x + width, y],
            color,
        });
        self.toolbar_vertices.push(UiVertex {
            position: [x + width, y + height],
            color,
        });
        self.toolbar_vertices.push(UiVertex {
            position: [x, y + height],
            color,
        });

        self.toolbar_indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    }

    fn add_rect_outline(&mut self, x: f32, y: f32, width: f32, height: f32, thickness: f32, color: [f32; 4]) {
        // Top
        self.add_rect(x, y + height - thickness, width, thickness, color);
        // Bottom
        self.add_rect(x, y, width, thickness, color);
        // Left
        self.add_rect(x, y, thickness, height, color);
        // Right
        self.add_rect(x + width - thickness, y, thickness, height, color);
    }

    pub fn get_crosshair_buffers(&self) -> (&[UiVertex], &[u32]) {
        (&self.crosshair_vertices, &self.crosshair_indices)
    }

    pub fn get_toolbar_buffers(&self) -> (&[UiVertex], &[u32]) {
        (&self.toolbar_vertices, &self.toolbar_indices)
    }

    #[allow(dead_code)]
    pub fn select_block(&mut self, slot: usize) {
        let block_types = [
            BlockType::Dirt,
            BlockType::Grass,
            BlockType::Sand,
            BlockType::Wood,
            BlockType::Planks,
            BlockType::Leaves,
            BlockType::Glass,
            BlockType::Water,
        ];
        
        if slot < block_types.len() {
            self.selected_block = block_types[slot];
            self.build_toolbar(); // Rebuild to update highlight
        }
    }

    pub fn next_block(&mut self) {
        let block_types = [
            BlockType::Dirt,
            BlockType::Grass,
            BlockType::Sand,
            BlockType::Wood,
            BlockType::Planks,
            BlockType::Leaves,
            BlockType::Glass,
            BlockType::Water,
        ];
        
        for (i, &block_type) in block_types.iter().enumerate() {
            if block_type == self.selected_block {
                let next_idx = (i + 1) % block_types.len();
                self.selected_block = block_types[next_idx];
                self.build_toolbar();
                return;
            }
        }
    }

    pub fn prev_block(&mut self) {
        let block_types = [
            BlockType::Dirt,
            BlockType::Grass,
            BlockType::Sand,
            BlockType::Wood,
            BlockType::Planks,
            BlockType::Leaves,
            BlockType::Glass,
            BlockType::Water,
        ];
        
        for (i, &block_type) in block_types.iter().enumerate() {
            if block_type == self.selected_block {
                let prev_idx = if i == 0 { block_types.len() - 1 } else { i - 1 };
                self.selected_block = block_types[prev_idx];
                self.build_toolbar();
                return;
            }
        }
    }
}
