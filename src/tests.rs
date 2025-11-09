#[cfg(test)]
mod tests {
    use crate::block::BlockType;
    use crate::chunk::Chunk;
    use crate::mesh::MeshBuilder;
    use crate::world::World;
    use crate::world_gen::WorldGenerator;

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
}
