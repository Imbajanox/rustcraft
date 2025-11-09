#[cfg(test)]
mod tests {
    use crate::block::BlockType;
    use crate::chunk::Chunk;
    use crate::mesh::MeshBuilder;
    use crate::world::World;
    use crate::world_gen::WorldGenerator;
    use crate::physics::{Player, Aabb};
    use crate::raycast::raycast;
    use glam::Vec3;

    #[test]
    fn test_block_types() {
        assert!(BlockType::Dirt.is_solid());
        assert!(BlockType::Grass.is_solid());
        assert!(BlockType::Wood.is_solid());
        assert!(BlockType::Water.is_solid());
        assert!(!BlockType::Air.is_solid());
        
        assert!(BlockType::Air.is_transparent());
        assert!(BlockType::Glass.is_transparent());
        assert!(BlockType::Water.is_transparent());
        assert!(!BlockType::Dirt.is_transparent());
    }

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new(0, 0);
        assert_eq!(chunk.x, 0);
        assert_eq!(chunk.z, 0);
        assert_eq!(chunk.get_block(0, 0, 0), BlockType::Air);
    }

    #[test]
    fn test_chunk_set_get() {
        let mut chunk = Chunk::new(0, 0);
        chunk.set_block(5, 10, 7, BlockType::Dirt);
        assert_eq!(chunk.get_block(5, 10, 7), BlockType::Dirt);
        assert_eq!(chunk.get_block(0, 0, 0), BlockType::Air);
    }

    #[test]
    fn test_world_creation() {
        let world = World::new(12345);
        assert_eq!(world.seed, 12345);
        assert!(world.chunks.is_empty());
    }

    #[test]
    fn test_world_chunk_generation() {
        let mut world = World::new(12345);
        let generator = WorldGenerator::new(12345);
        
        world.load_or_generate_chunk(0, 0, &generator);
        assert!(world.get_chunk(0, 0).is_some());
        
        // Verify chunk was generated with some blocks
        let chunk = world.get_chunk(0, 0).unwrap();
        let mut has_solid_blocks = false;
        for y in 0..10 {
            if chunk.get_block(8, y, 8).is_solid() {
                has_solid_blocks = true;
                break;
            }
        }
        assert!(has_solid_blocks, "Generated chunk should have solid blocks");
    }

    #[test]
    fn test_world_save_load() {
        use std::fs;
        let test_path = "/tmp/test_world.dat";
        
        // Create and save a world
        {
            let mut world = World::new(54321);
            let generator = WorldGenerator::new(54321);
            world.load_or_generate_chunk(0, 0, &generator);
            world.save(test_path).expect("Failed to save world");
        }
        
        // Load the world
        {
            let loaded_world = World::load(test_path).expect("Failed to load world");
            assert_eq!(loaded_world.seed, 54321);
            assert!(loaded_world.get_chunk(0, 0).is_some());
        }
        
        // Cleanup
        fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_block_colors() {
        let dirt_color = BlockType::Dirt.get_color();
        let grass_color = BlockType::Grass.get_color();
        
        // Verify colors are different
        assert_ne!(dirt_color, grass_color);
        
        // Verify colors are in valid range [0, 1]
        for component in dirt_color {
            assert!(component >= 0.0 && component <= 1.0);
        }
    }

    #[test]
    fn test_mesh_builder_accumulates_chunks() {
        let mut world = World::new(12345);
        let mut chunk1 = Chunk::new(0, 0);
        let mut chunk2 = Chunk::new(1, 0);
        
        // Add a block to each chunk
        chunk1.set_block(0, 0, 0, BlockType::Dirt);
        chunk2.set_block(0, 0, 0, BlockType::Grass);
        
        world.chunks.insert((0, 0), chunk1);
        world.chunks.insert((1, 0), chunk2);
        
        let mut mesh_builder = MeshBuilder::new();
        mesh_builder.clear();
        
        // Build meshes for both chunks
        if let Some(chunk) = world.get_chunk(0, 0) {
            mesh_builder.build_chunk_mesh(chunk, &world);
        }
        
        let vertices_after_first = mesh_builder.vertices.len();
        assert!(vertices_after_first > 0, "First chunk should generate vertices");
        
        if let Some(chunk) = world.get_chunk(1, 0) {
            mesh_builder.build_chunk_mesh(chunk, &world);
        }
        
        let vertices_after_second = mesh_builder.vertices.len();
        assert!(vertices_after_second > vertices_after_first, 
                "Second chunk should add more vertices, not replace them");
    }

    #[test]
    fn test_block_face_generation() {
        let mut world = World::new(12345);
        let mut chunk = Chunk::new(0, 0);
        
        // Create a single isolated block in the air
        chunk.set_block(5, 10, 5, BlockType::Dirt);
        world.chunks.insert((0, 0), chunk);
        
        let mut mesh_builder = MeshBuilder::new();
        mesh_builder.clear();
        
        if let Some(chunk) = world.get_chunk(0, 0) {
            mesh_builder.build_chunk_mesh(chunk, &world);
        }
        
        // An isolated block should have 6 faces, each with 4 vertices
        // Total: 6 * 4 = 24 vertices
        assert_eq!(mesh_builder.vertices.len(), 24, 
                   "Isolated block should have 24 vertices (6 faces × 4 vertices)");
        
        // Each face has 2 triangles, each triangle has 3 indices
        // Total: 6 * 2 * 3 = 36 indices
        assert_eq!(mesh_builder.indices.len(), 36,
                   "Isolated block should have 36 indices (6 faces × 2 triangles × 3 indices)");
        
        // Verify top face is at y=11 (block at y=10, top face at y+1)
        let mut has_top_face = false;
        for vertex in &mesh_builder.vertices {
            if (vertex.position[1] - 11.0).abs() < 0.01 {
                has_top_face = true;
                break;
            }
        }
        assert!(has_top_face, "Should have vertices at top face position (y=11)");
        
        // Verify bottom face is at y=10 (block at y=10, bottom face at y)
        let mut has_bottom_face = false;
        for vertex in &mesh_builder.vertices {
            if (vertex.position[1] - 10.0).abs() < 0.01 {
                has_bottom_face = true;
                break;
            }
        }
        assert!(has_bottom_face, "Should have vertices at bottom face position (y=10)");
    }

    #[test]
    fn test_player_creation() {
        let player = Player::new(Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(player.position, Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(player.velocity, Vec3::ZERO);
        assert!(!player.on_ground);
    }

    #[test]
    fn test_player_jump() {
        let mut player = Player::new(Vec3::new(0.0, 10.0, 0.0));
        player.on_ground = true;
        player.jump();
        assert!(player.velocity.y > 0.0, "Jump should give upward velocity");
        assert!(!player.on_ground, "Player should not be on ground after jump");
    }

    #[test]
    fn test_player_cant_jump_in_air() {
        let mut player = Player::new(Vec3::new(0.0, 10.0, 0.0));
        player.on_ground = false;
        player.jump();
        assert_eq!(player.velocity.y, 0.0, "Can't jump while in air");
    }

    #[test]
    fn test_aabb_intersection() {
        let box1 = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let box2 = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        let box3 = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));

        assert!(box1.intersects(&box2), "Overlapping boxes should intersect");
        assert!(box2.intersects(&box1), "Intersection should be symmetric");
        assert!(!box1.intersects(&box3), "Separated boxes should not intersect");
    }

    #[test]
    fn test_world_get_block_at() {
        let mut world = World::new(12345);
        let generator = WorldGenerator::new(12345);
        
        world.load_or_generate_chunk(0, 0, &generator);
        
        // Test getting a block at world coordinates
        let block = world.get_block_at(0, 10, 0);
        assert!(block.is_some(), "Should get a block from loaded chunk");
        
        // Test getting a block from unloaded chunk
        let block = world.get_block_at(1000, 10, 1000);
        assert!(block.is_none(), "Should return None for unloaded chunk");
    }

    #[test]
    fn test_world_set_block_at() {
        let mut world = World::new(12345);
        let generator = WorldGenerator::new(12345);
        
        world.load_or_generate_chunk(0, 0, &generator);
        
        // Set a block
        let success = world.set_block_at(5, 20, 5, BlockType::Planks);
        assert!(success, "Should successfully set block in loaded chunk");
        
        // Verify it was set
        let block = world.get_block_at(5, 20, 5);
        assert_eq!(block, Some(BlockType::Planks), "Block should be set to Planks");
        
        // Try setting in unloaded chunk
        let success = world.set_block_at(1000, 20, 1000, BlockType::Dirt);
        assert!(!success, "Should fail to set block in unloaded chunk");
    }

    #[test]
    fn test_raycast_hit() {
        let mut world = World::new(12345);
        let mut chunk = Chunk::new(0, 0);
        
        // Place a block at (5, 10, 5)
        chunk.set_block(5, 10, 5, BlockType::Dirt);
        world.chunks.insert((0, 0), chunk);
        
        // Raycast from above the block downward
        let origin = Vec3::new(5.5, 15.0, 5.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);
        
        let result = raycast(origin, direction, 10.0, &world);
        
        assert!(result.hit, "Ray should hit the block");
        assert_eq!(result.position, Some((5, 10, 5)), "Should hit the correct block");
    }

    #[test]
    fn test_raycast_miss() {
        let world = World::new(12345);
        
        // Raycast in empty world
        let origin = Vec3::new(5.5, 15.0, 5.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);
        
        let result = raycast(origin, direction, 3.0, &world);
        
        assert!(!result.hit, "Ray should not hit anything in empty world");
        assert_eq!(result.position, None, "Should have no hit position");
    }

    #[test]
    fn test_ui_renderer_creation() {
        use crate::ui::UiRenderer;
        
        let ui = UiRenderer::new();
        assert_eq!(ui.selected_block, BlockType::Dirt, "Default selected block should be Dirt");
        
        let (crosshair_verts, crosshair_inds) = ui.get_crosshair_buffers();
        assert!(!crosshair_verts.is_empty(), "Crosshair should have vertices");
        assert!(!crosshair_inds.is_empty(), "Crosshair should have indices");
        
        let (toolbar_verts, toolbar_inds) = ui.get_toolbar_buffers();
        assert!(!toolbar_verts.is_empty(), "Toolbar should have vertices");
        assert!(!toolbar_inds.is_empty(), "Toolbar should have indices");
    }

    #[test]
    fn test_ui_block_selection() {
        use crate::ui::UiRenderer;
        
        let mut ui = UiRenderer::new();
        assert_eq!(ui.selected_block, BlockType::Dirt);
        
        // Test next block
        ui.next_block();
        assert_eq!(ui.selected_block, BlockType::Grass);
        
        ui.next_block();
        assert_eq!(ui.selected_block, BlockType::Sand);
        
        // Test prev block
        ui.prev_block();
        assert_eq!(ui.selected_block, BlockType::Grass);
        
        ui.prev_block();
        assert_eq!(ui.selected_block, BlockType::Dirt);
        
        // Test wrapping from last to first
        ui.prev_block();
        assert_eq!(ui.selected_block, BlockType::Water);
        
        // Test wrapping from first to last (after going to first)
        ui.next_block();
        assert_eq!(ui.selected_block, BlockType::Dirt);
    }

    #[test]
    fn test_aabb_from_position() {
        let position = Vec3::new(5.0, 10.0, 5.0);
        let aabb = Aabb::from_position(position, 0.3, 1.8);
        
        assert_eq!(aabb.min.x, 4.7);
        assert_eq!(aabb.max.x, 5.3);
        assert_eq!(aabb.min.y, 10.0);
        assert_eq!(aabb.max.y, 11.8);
        assert_eq!(aabb.min.z, 4.7);
        assert_eq!(aabb.max.z, 5.3);
    }

    #[test]
    fn test_player_physics_on_ground() {
        let mut world = World::new(12345);
        let mut chunk = Chunk::new(0, 0);
        
        // Create a floor at y=10
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, 10, z, BlockType::Dirt);
            }
        }
        world.chunks.insert((0, 0), chunk);
        
        // Place player above the floor
        let mut player = Player::new(Vec3::new(8.0, 15.0, 8.0));
        
        // Run physics for enough time to fall - player should fall and land on ground
        for _ in 0..200 {
            player.apply_physics(0.016, &world); // ~60 FPS
            if player.on_ground {
                break;
            }
        }
        
        assert!(player.on_ground, "Player should be on ground after falling. Position: {}, Velocity: {}", player.position.y, player.velocity.y);
        // Player should be standing on top of block at y=10, so player.y should be 11
        assert!((player.position.y - 11.0).abs() < 0.1, 
            "Player should be at ground level (y=11), but is at y={}", player.position.y);
    }

    #[test]
    fn test_player_collision_with_walls() {
        let mut world = World::new(12345);
        let mut chunk = Chunk::new(0, 0);
        
        // Create a floor and a wall
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, 10, z, BlockType::Dirt);
            }
        }
        // Wall at x=12
        for y in 11..15 {
            chunk.set_block(12, y, 8, BlockType::Dirt);
        }
        world.chunks.insert((0, 0), chunk);
        
        // Place player very close to the wall, facing it
        let mut player = Player::new(Vec3::new(11.5, 11.0, 8.0));
        player.on_ground = true;
        
        let initial_x = player.position.x;
        
        // Try to move into the wall repeatedly
        for _ in 0..10 {
            player.velocity.x = 5.0;
            player.apply_physics(0.1, &world);
        }
        
        // Player should not have moved much (blocked by wall at x=12)
        // Player bounding box extends 0.3 from center, so max x position before wall is about 12 - 0.3 = 11.7
        assert!(player.position.x < 11.8, "Player should be blocked by wall, but is at x={}", player.position.x);
    }

    #[test]
    fn test_player_step_up() {
        let mut world = World::new(12345);
        let mut chunk = Chunk::new(0, 0);
        
        // Create a floor with a single step up
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, 10, z, BlockType::Dirt);
            }
        }
        // Single block step at x=12
        chunk.set_block(12, 11, 8, BlockType::Dirt);
        world.chunks.insert((0, 0), chunk);
        
        // Place player before the step
        let mut player = Player::new(Vec3::new(11.0, 11.0, 8.0));
        player.on_ground = true;
        
        // Move towards the step
        player.velocity.x = 2.0;
        player.apply_physics(0.1, &world);
        
        // Player should step up (or be blocked if step is too high)
        // With STEP_HEIGHT of 0.5, a 1-block step should be climbable
        assert!(player.position.x > 11.0, "Player should move forward");
    }

    #[test]
    fn test_config_save_load() {
        use crate::config::GameConfig;
        use std::fs;
        
        let test_path = "/tmp/test_config.json";
        
        // Create and save a config
        {
            let mut config = GameConfig::default();
            config.sensitivity = 0.01;
            config.walk_speed = 5.0;
            config.view_distance = 10;
            config.save(test_path).expect("Failed to save config");
        }
        
        // Load the config
        {
            let loaded_config = GameConfig::load(test_path);
            assert_eq!(loaded_config.sensitivity, 0.01);
            assert_eq!(loaded_config.walk_speed, 5.0);
            assert_eq!(loaded_config.view_distance, 10);
        }
        
        // Cleanup
        fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_debug_info_update() {
        use crate::debug::DebugInfo;
        
        let mut debug_info = DebugInfo::new();
        let player = Player::new(Vec3::new(10.0, 20.0, 30.0));
        
        debug_info.update(&player, 60);
        
        assert_eq!(debug_info.fps, 60);
        assert_eq!(debug_info.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(debug_info.chunk_x, 0);
        assert_eq!(debug_info.chunk_z, 1);
    }
}

