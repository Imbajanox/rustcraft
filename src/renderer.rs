use crate::camera::Camera;
use crate::mesh::MeshBuilder;
use crate::ui::{UiRenderer, UiVertex};
use crate::vertex::{Uniforms, Vertex};
use crate::world::World;
use wgpu::util::DeviceExt;
use std::collections::HashMap;

fn load_texture_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<(wgpu::Texture, wgpu::TextureView, wgpu::Sampler), String> {
    // Try to load texture atlas from textures directory
    let texture_bytes = match std::fs::read("textures/atlas.png") {
        Ok(bytes) => bytes,
        Err(_) => {
            // Try individual texture as fallback
            match std::fs::read("textures/dirt.png") {
                Ok(bytes) => bytes,
                Err(_) => {
                    // If texture loading fails, create a simple 16x16 white texture as fallback
                    return create_fallback_texture(device, queue);
                }
            }
        }
    };

    let img = match image::load_from_memory(&texture_bytes) {
        Ok(img) => img.to_rgba8(),
        Err(_) => return create_fallback_texture(device, queue),
    };

    let dimensions = img.dimensions();
    let size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Block Texture Atlas"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Ok((texture, view, sampler))
}

fn create_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<(wgpu::Texture, wgpu::TextureView, wgpu::Sampler), String> {
    // Create a simple 16x16 white texture
    let size = wgpu::Extent3d {
        width: 16,
        height: 16,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Fallback Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Create white pixels
    let pixels: Vec<u8> = (0..16 * 16)
        .flat_map(|_| [255u8, 255u8, 255u8, 255u8])
        .collect();

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * 16),
            rows_per_image: Some(16),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Ok((texture, view, sampler))
}

pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    ui_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    uniforms: Uniforms,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32,
    crosshair_vertex_buffer: Option<wgpu::Buffer>,
    crosshair_index_buffer: Option<wgpu::Buffer>,
    crosshair_num_indices: u32,
    toolbar_vertex_buffer: Option<wgpu::Buffer>,
    toolbar_index_buffer: Option<wgpu::Buffer>,
    toolbar_num_indices: u32,
    chunk_mesh_cache: HashMap<(i32, i32), ChunkMesh>,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniforms = Uniforms::new();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        // Load texture
        let (_texture, texture_view, texture_sampler) = 
            load_texture_atlas(&device, &queue).unwrap_or_else(|_| {
                create_fallback_texture(&device, &queue).unwrap()
            });

        // Create texture bind group layout
        let texture_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create UI pipeline
        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_shader.wgsl").into()),
        });

        let ui_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&ui_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: "vs_main",
                buffers: &[UiVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for UI
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            // IMPORTANT: UI is drawn into the same render pass that has a depth-stencil attachment.
            // All pipelines used with a render pass that includes a depth-stencil must specify a
            // compatible depth-stencil format. The UI doesn't need depth testing, so we set depth
            // writes off and use CompareFunction::Always so it doesn't discard fragments.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            ui_pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group,
            uniforms,
            depth_texture,
            depth_view,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            crosshair_vertex_buffer: None,
            crosshair_index_buffer: None,
            crosshair_num_indices: 0,
            toolbar_vertex_buffer: None,
            toolbar_index_buffer: None,
            toolbar_num_indices: 0,
            chunk_mesh_cache: HashMap::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn update_mesh(&mut self, world: &mut World, camera: &Camera, view_distance: i32) {
        let cam_chunk_x = (camera.position.x / 16.0).floor() as i32;
        let cam_chunk_z = (camera.position.z / 16.0).floor() as i32;

        let render_distance = view_distance;
        
        // Evict chunks from cache that are too far away (beyond render distance + buffer)
        let eviction_distance = render_distance + 2;
        self.chunk_mesh_cache.retain(|&(chunk_x, chunk_z), _| {
            let dx = (chunk_x - cam_chunk_x).abs();
            let dz = (chunk_z - cam_chunk_z).abs();
            dx <= eviction_distance && dz <= eviction_distance
        });
        
        // Build or update chunk meshes for dirty chunks
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let chunk_x = cam_chunk_x + dx;
                let chunk_z = cam_chunk_z + dz;
                let chunk_key = (chunk_x, chunk_z);

                if let Some(chunk) = world.get_chunk(chunk_x, chunk_z) {
                    // Only rebuild mesh if chunk is dirty or not cached
                    if chunk.dirty || !self.chunk_mesh_cache.contains_key(&chunk_key) {
                        let mut mesh_builder = MeshBuilder::new();
                        mesh_builder.build_chunk_mesh(chunk, world);
                        
                        self.chunk_mesh_cache.insert(chunk_key, ChunkMesh {
                            vertices: mesh_builder.vertices,
                            indices: mesh_builder.indices,
                        });
                    }
                }
            }
        }
        
        // Mark all visible chunks as clean
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let chunk_x = cam_chunk_x + dx;
                let chunk_z = cam_chunk_z + dz;
                if let Some(chunk) = world.get_chunk_mut(chunk_x, chunk_z) {
                    chunk.mark_clean();
                }
            }
        }
        
        // Combine all visible chunk meshes into single buffers
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        
        for dx in -render_distance..=render_distance {
            for dz in -render_distance..=render_distance {
                let chunk_x = cam_chunk_x + dx;
                let chunk_z = cam_chunk_z + dz;
                let chunk_key = (chunk_x, chunk_z);
                
                if let Some(chunk_mesh) = self.chunk_mesh_cache.get(&chunk_key) {
                    let vertex_offset = all_vertices.len() as u32;
                    all_vertices.extend_from_slice(&chunk_mesh.vertices);
                    
                    // Offset indices by current vertex count
                    for &index in &chunk_mesh.indices {
                        all_indices.push(index + vertex_offset);
                    }
                }
            }
        }

        if !all_vertices.is_empty() {
            self.vertex_buffer = Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&all_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            );

            self.index_buffer = Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&all_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            );

            self.num_indices = all_indices.len() as u32;
        }
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.uniforms
            .update_view_proj(camera.get_view_matrix(), camera.get_projection_matrix());
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn update_ui(&mut self, ui: &UiRenderer) {
        // Update crosshair buffers
        let (crosshair_verts, crosshair_inds) = ui.get_crosshair_buffers();
        if !crosshair_verts.is_empty() {
            self.crosshair_vertex_buffer = Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Crosshair Vertex Buffer"),
                        contents: bytemuck::cast_slice(crosshair_verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            );
            self.crosshair_index_buffer = Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Crosshair Index Buffer"),
                        contents: bytemuck::cast_slice(crosshair_inds),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            );
            self.crosshair_num_indices = crosshair_inds.len() as u32;
        }

        // Update toolbar buffers
        let (toolbar_verts, toolbar_inds) = ui.get_toolbar_buffers();
        if !toolbar_verts.is_empty() {
            self.toolbar_vertex_buffer = Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Toolbar Vertex Buffer"),
                        contents: bytemuck::cast_slice(toolbar_verts),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            );
            self.toolbar_index_buffer = Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Toolbar Index Buffer"),
                        contents: bytemuck::cast_slice(toolbar_inds),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
            );
            self.toolbar_num_indices = toolbar_inds.len() as u32;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Render world
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);

            if let (Some(vertex_buffer), Some(index_buffer)) =
                (&self.vertex_buffer, &self.index_buffer)
            {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            // Render UI elements
            render_pass.set_pipeline(&self.ui_pipeline);

            // Render toolbar
            if let (Some(vertex_buffer), Some(index_buffer)) =
                (&self.toolbar_vertex_buffer, &self.toolbar_index_buffer)
            {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.toolbar_num_indices, 0, 0..1);
            }

            // Render crosshair
            if let (Some(vertex_buffer), Some(index_buffer)) =
                (&self.crosshair_vertex_buffer, &self.crosshair_index_buffer)
            {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.crosshair_num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
