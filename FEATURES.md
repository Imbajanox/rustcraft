# Rustcraft - Feature Documentation

## Implemented Features

### ✅ Block Types (7 Total)
All 7 required block types have been implemented:

1. **Dirt** - Brown blocks forming the underground
2. **Sand** - Beige blocks found near water level
3. **Grass** - Green blocks covering the surface
4. **Wood** - Brown log blocks forming tree trunks
5. **Leaves** - Dark green blocks forming tree canopies
6. **Planks** - Orange-brown blocks (available for future use)
7. **Glass** - Semi-transparent light blue blocks (available for future use)

Each block type has:
- Unique color representation
- Solid/transparent properties
- Proper rendering with face culling

### ✅ Procedural World Generation
The world generation system includes:

- **Perlin Noise-based Terrain**: Multi-octave noise for natural-looking landscapes
- **Varied Height Maps**: Hills, valleys, and plains generated procedurally
- **Biome-like Features**: Different terrain types based on height
  - Sandy beaches at low elevations
  - Grassy plains at normal elevations
  - Varied elevation from Y=30 to Y=50
- **Tree Generation**: Randomly placed trees with:
  - Wood trunk (4 blocks tall)
  - Leaf canopy (3x3x3 area at top)
  - Natural spacing using noise
- **Infinite World**: Chunks generate on-demand as you explore
- **Deterministic**: Same seed always generates same world

### ✅ World Saving and Loading
Complete world persistence:

- **Binary Format**: Efficient storage using bincode
- **File Location**: `world.dat` in project root
- **Auto-save**: World saves automatically when exiting (press Escape)
- **Auto-load**: Saved world loads automatically on startup
- **Chunk Serialization**: All chunks and their blocks are saved
- **Seed Preservation**: World seed is saved and restored

### ✅ First-Person Camera and Movement
Full 3D first-person controls:

- **Camera System**:
  - Free-look camera with mouse control
  - Adjustable field of view (70°)
  - Proper view and projection matrices
  - Pitch clamping to prevent camera flipping

- **Movement Controls**:
  - W/A/S/D for horizontal movement
  - Space for upward movement
  - Left Shift for downward movement
  - Smooth movement with delta time
  - 10 units/second movement speed

- **Mouse Look**:
  - Hold left mouse button to look around
  - Smooth mouse sensitivity
  - Yaw (left/right) rotation
  - Pitch (up/down) rotation with limits

## Technical Implementation

### Architecture
- **Modular Design**: Separated into logical modules
  - `block.rs` - Block type definitions
  - `chunk.rs` - Chunk data structure
  - `world.rs` - World management
  - `world_gen.rs` - Procedural generation
  - `camera.rs` - Camera and view matrices
  - `input.rs` - Input handling
  - `mesh.rs` - Mesh generation
  - `renderer.rs` - Graphics rendering
  - `vertex.rs` - Vertex definitions

### Rendering System
- **Graphics API**: wgpu (WebGPU) for cross-platform support
- **Chunk Meshing**: Efficient mesh generation with:
  - Hidden face culling (only visible faces rendered)
  - Greedy meshing potential
  - Per-face shading for depth perception
- **Depth Testing**: Proper 3D depth rendering
- **Sky Color**: Pleasant light blue sky (RGB: 0.53, 0.81, 0.92)
- **Render Distance**: 3 chunks in each direction (7x7 chunk area)

### Performance Optimizations
- **Chunk System**: 16x16x64 block chunks for memory efficiency
- **Face Culling**: Only render visible block faces
- **On-demand Generation**: Chunks generate only when needed
- **Binary Serialization**: Fast save/load with bincode
- **Release Mode**: Optimized compilation for better FPS

### Code Quality
- **Tests**: 7 unit tests covering core functionality
- **Clippy Clean**: No warnings from Rust linter
- **Type Safety**: Full Rust type safety
- **Error Handling**: Proper error propagation

## File Structure
```
rustcraft/
├── Cargo.toml              # Dependencies and project config
├── README.md               # Main documentation
├── WINDOWS_GUIDE.md        # Detailed Windows instructions
├── world.dat               # Saved world (generated at runtime)
└── src/
    ├── main.rs             # Entry point and event loop
    ├── block.rs            # Block type enum and properties
    ├── chunk.rs            # Chunk data structure
    ├── world.rs            # World state and serialization
    ├── world_gen.rs        # Procedural generation logic
    ├── camera.rs           # Camera and matrices
    ├── input.rs            # Keyboard and mouse input
    ├── mesh.rs             # Chunk mesh generation
    ├── renderer.rs         # wgpu rendering system
    ├── vertex.rs           # Vertex and uniform definitions
    ├── shader.wgsl         # WGSL shader code
    └── tests.rs            # Unit tests
```

## Statistics
- **Total Lines of Code**: ~1,170 lines of Rust
- **Number of Modules**: 12
- **Number of Tests**: 32
- **Dependencies**: 8 main crates
- **Compilation Time**: ~1-2 minutes (first build)
- **Binary Size**: ~5-10 MB (release build)

## Recent Enhancements

### ✅ Inventory System (v0.2.0)
- Complete Minecraft-like inventory system implemented
- 9-slot toolbar for quick access
- 27-slot storage for additional items
- Item stacking with max stack size of 64
- Destroyed blocks automatically added to inventory
- Placing blocks removes items from inventory
- Visual UI with item count indicators
- Press 'E' to toggle full inventory panel
- Mouse wheel to select toolbar slots
- Inventory persists with world save/load
- 7 comprehensive tests for inventory functionality

## Future Enhancement Possibilities
While the core features are implemented, the codebase supports:
- ~~Block placement/destruction~~ ✅ Implemented
- ~~Inventory system~~ ✅ Implemented
- **Drag-and-drop inventory management** (Planned)
- More block types
- Improved terrain generation
- Multiplayer support
- Texture mapping
- Lighting system
- Crafting mechanics
- Hotbar number key selection (1-9)
