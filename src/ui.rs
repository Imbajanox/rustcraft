use bytemuck::{Pod, Zeroable};
use crate::block::BlockType;
use crate::inventory::Inventory;

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
    inventory_open: bool,
    inventory_vertices: Vec<UiVertex>,
    inventory_indices: Vec<u32>,
}

impl UiRenderer {
    pub fn new() -> Self {
        let mut ui = Self {
            selected_block: BlockType::Dirt,
            crosshair_vertices: Vec::new(),
            crosshair_indices: Vec::new(),
            toolbar_vertices: Vec::new(),
            toolbar_indices: Vec::new(),
            inventory_open: false,
            inventory_vertices: Vec::new(),
            inventory_indices: Vec::new(),
        };
        ui.build_crosshair();
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

    pub fn build_toolbar(&mut self, inventory: &Inventory) {
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

        // Draw slot borders and contents
        for i in 0..num_slots {
            let x = -toolbar_width / 2.0 + i as f32 * slot_size;
            let border_color = [0.8, 0.8, 0.8, 0.8];
            
            // Draw border as outline
            self.add_rect_outline(x, y_pos, slot_size, toolbar_height, border_thickness, border_color);

            // Draw item in slot if present
            if let Some(stack) = &inventory.toolbar[i] {
                let padding = slot_size * 0.2;
                let block_size = slot_size - 2.0 * padding;
                let color = stack.block_type.get_color();
                let block_color = [color[0], color[1], color[2], 1.0];
                
                self.add_rect(
                    x + padding,
                    y_pos + padding,
                    block_size,
                    toolbar_height - 2.0 * padding,
                    block_color,
                );

                // Draw item count if > 1
                if stack.count > 1 {
                    // We'll draw a small indicator for count
                    // For now, just make a small white rectangle to indicate multiple items
                    let count_indicator_size = slot_size * 0.15;
                    let count_color = [1.0, 1.0, 1.0, 0.8];
                    self.add_rect(
                        x + slot_size - padding - count_indicator_size,
                        y_pos + padding,
                        count_indicator_size,
                        count_indicator_size,
                        count_color,
                    );
                }
            }
        }

        // Highlight selected slot
        let x = -toolbar_width / 2.0 + inventory.selected_slot as f32 * slot_size;
        let highlight_color = [1.0, 1.0, 1.0, 1.0];
        self.add_rect_outline(x, y_pos, slot_size, toolbar_height, border_thickness * 2.0, highlight_color);
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

    pub fn get_inventory_buffers(&self) -> (&[UiVertex], &[u32]) {
        (&self.inventory_vertices, &self.inventory_indices)
    }

    pub fn is_inventory_open(&self) -> bool {
        self.inventory_open
    }

    pub fn toggle_inventory(&mut self) {
        self.inventory_open = !self.inventory_open;
    }

    pub fn build_inventory(&mut self, inventory: &Inventory) {
        self.inventory_vertices.clear();
        self.inventory_indices.clear();

        if !self.inventory_open {
            return;
        }

        // Inventory panel dimensions
        let panel_width = 0.8;
        let panel_height = 0.6;
        let slot_size = 0.07;
        let slot_gap = 0.005;
        let border_thickness = 0.003;

        // Center the panel
        let panel_x = -panel_width / 2.0;
        let panel_y = -panel_height / 2.0;

        // Draw semi-transparent background
        let bg_color = [0.0, 0.0, 0.0, 0.8];
        self.add_inventory_rect(panel_x, panel_y, panel_width, panel_height, bg_color);

        // Draw title area
        let title_height = 0.08;
        let title_color = [0.2, 0.2, 0.2, 0.9];
        self.add_inventory_rect(panel_x, panel_y + panel_height - title_height, panel_width, title_height, title_color);

        // Draw storage slots (3 rows of 9)
        let start_x = panel_x + 0.1;
        let start_y = panel_y + panel_height - title_height - 0.15;

        for row in 0..3 {
            for col in 0..9 {
                let slot_idx = row * 9 + col;
                let x = start_x + col as f32 * (slot_size + slot_gap);
                let y = start_y - row as f32 * (slot_size + slot_gap);

                // Draw slot background
                let slot_bg = [0.3, 0.3, 0.3, 0.9];
                self.add_inventory_rect(x, y, slot_size, slot_size, slot_bg);

                // Draw slot border
                let border_color = [0.5, 0.5, 0.5, 1.0];
                self.add_inventory_rect_outline(x, y, slot_size, slot_size, border_thickness, border_color);

                // Draw item if present
                if let Some(stack) = &inventory.storage[slot_idx] {
                    let padding = slot_size * 0.15;
                    let item_size = slot_size - 2.0 * padding;
                    let color = stack.block_type.get_color();
                    let item_color = [color[0], color[1], color[2], 1.0];
                    
                    self.add_inventory_rect(
                        x + padding,
                        y + padding,
                        item_size,
                        item_size,
                        item_color,
                    );

                    // Draw count indicator if > 1
                    if stack.count > 1 {
                        let count_size = slot_size * 0.15;
                        let count_color = [1.0, 1.0, 1.0, 0.9];
                        self.add_inventory_rect(
                            x + slot_size - padding - count_size,
                            y + padding,
                            count_size,
                            count_size,
                            count_color,
                        );
                    }
                }
            }
        }

        // Draw toolbar slots at bottom (same as in build_toolbar but in panel)
        let toolbar_y = panel_y + 0.05;
        for i in 0..9 {
            let x = start_x + i as f32 * (slot_size + slot_gap);

            // Draw slot background
            let slot_bg = [0.3, 0.3, 0.3, 0.9];
            self.add_inventory_rect(x, toolbar_y, slot_size, slot_size, slot_bg);

            // Draw slot border
            let border_color = if i == inventory.selected_slot {
                [1.0, 1.0, 1.0, 1.0] // Highlight selected slot
            } else {
                [0.5, 0.5, 0.5, 1.0]
            };
            let thickness = if i == inventory.selected_slot {
                border_thickness * 2.0
            } else {
                border_thickness
            };
            self.add_inventory_rect_outline(x, toolbar_y, slot_size, slot_size, thickness, border_color);

            // Draw item if present
            if let Some(stack) = &inventory.toolbar[i] {
                let padding = slot_size * 0.15;
                let item_size = slot_size - 2.0 * padding;
                let color = stack.block_type.get_color();
                let item_color = [color[0], color[1], color[2], 1.0];
                
                self.add_inventory_rect(
                    x + padding,
                    toolbar_y + padding,
                    item_size,
                    item_size,
                    item_color,
                );

                // Draw count indicator if > 1
                if stack.count > 1 {
                    let count_size = slot_size * 0.15;
                    let count_color = [1.0, 1.0, 1.0, 0.9];
                    self.add_inventory_rect(
                        x + slot_size - padding - count_size,
                        toolbar_y + padding,
                        count_size,
                        count_size,
                        count_color,
                    );
                }
            }
        }
    }

    fn add_inventory_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        let base_idx = self.inventory_vertices.len() as u32;

        self.inventory_vertices.push(UiVertex {
            position: [x, y],
            color,
        });
        self.inventory_vertices.push(UiVertex {
            position: [x + width, y],
            color,
        });
        self.inventory_vertices.push(UiVertex {
            position: [x + width, y + height],
            color,
        });
        self.inventory_vertices.push(UiVertex {
            position: [x, y + height],
            color,
        });

        self.inventory_indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    }

    fn add_inventory_rect_outline(&mut self, x: f32, y: f32, width: f32, height: f32, thickness: f32, color: [f32; 4]) {
        // Top
        self.add_inventory_rect(x, y + height - thickness, width, thickness, color);
        // Bottom
        self.add_inventory_rect(x, y, width, thickness, color);
        // Left
        self.add_inventory_rect(x, y, thickness, height, color);
        // Right
        self.add_inventory_rect(x + width - thickness, y, thickness, height, color);
    }

    #[allow(dead_code)]
    pub fn select_block(&mut self, slot: usize) {
        if slot < 9 {
            self.selected_block = BlockType::Dirt; // Will be overridden by inventory
        }
    }

    pub fn sync_selected_block(&mut self, inventory: &Inventory) {
        if let Some(block_type) = inventory.get_selected_block() {
            self.selected_block = block_type;
        }
    }
}
