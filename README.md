This will be a Minecraft like Voxel game made in Rust

# Rustcraft - Voxel Game Prototype

A Minecraft-like voxel game prototype built in Rust with procedural world generation, multiple block types, and first-person controls.

## Features

- **7 Block Types**: Dirt, Sand, Grass, Wood, Leaves, Planks, and Glass
- **Procedural World Generation**: Infinite terrain with hills, valleys, and trees
- **World Saving**: Automatically saves world state when exiting
- **First-Person Camera**: WASD movement with physics-based controls
- **Physics System**: Gravity, jumping, and collision detection
- **Block Interaction**: Place and destroy blocks with mouse clicks
- **Chunk-based Rendering**: Efficient rendering with culling of hidden faces

## Controls

- **W/A/S/D**: Move forward/left/backward/right
- **Space**: Jump (when on ground)
- **Mouse Movement**: Look around (cursor is automatically grabbed)
- **Mouse Wheel**: Select block type to place
- **Left Mouse Button**: Destroy block
- **Right Mouse Button**: Place block
- **F3**: Toggle debug mode (shows detailed info in console)
- **Escape**: Save and quit

## Configuration

The game creates a `config.json` file on first run with the following configurable settings:

- `sensitivity`: Mouse look sensitivity (default: 0.005)
- `walk_speed`: Player movement speed in blocks/second (default: 4.3)
- `view_distance`: How many chunks to render around the player (default: 6)
- `fov`: Field of view in degrees (default: 70.0)
- `show_debug`: Whether to show debug info by default (default: false)

You can edit this file to customize your game settings. Changes are saved when you exit the game.

## Running on Windows

### Prerequisites

1. **Install Rust**: Download and install from [https://rustup.rs/](https://rustup.rs/)
   - The installer will set up `cargo` and `rustc` automatically
   - You may need to restart your terminal after installation

2. **Install Visual Studio Build Tools** (if not already installed):
   - Download from [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/)
   - Select "Desktop development with C++" workload
   - This is required for the Rust compiler on Windows

### Building and Running

1. **Open a terminal** (Command Prompt, PowerShell, or Windows Terminal)

2. **Navigate to the project directory**:
   ```
   cd path\to\rustcraft
   ```

3. **Build the project**:
   ```
   cargo build --release
   ```
   This will take a few minutes the first time as it downloads and compiles dependencies.

4. **Run the game**:
   ```
   cargo run --release
   ```

   Or directly run the compiled executable:
   ```
   target\release\rustcraft.exe
   ```

### Notes

- The game will create a `world.dat` file to save your world state
- A `config.json` file will be created to store your settings
- Performance is better in release mode (with `--release` flag)
- First run may take longer as it generates the initial world chunks
- Console will show FPS and current position (press F3 for detailed debug info)
- The world and config will be saved automatically when you press Escape or close the window

## Recent Improvements

### Enhanced Collision Detection (v0.1.0)
- Fixed issues with player falling through blocks at edges and corners
- Improved collision handling with epsilon margins for more stable physics
- Added step-up mechanics allowing players to climb blocks up to 0.5 units high
- Better ground and ceiling collision detection
- Smoother player movement along walls

### Configuration System
- Customizable game settings via `config.json`
- Adjustable mouse sensitivity, movement speed, and view distance
- Settings persist between game sessions

### Debug Mode
- Press F3 to toggle enhanced debug information
- Shows detailed player stats: position, velocity, chunk coordinates
- Displays FPS counter and ground state

## Technical Details

- **Rendering**: Uses wgpu (WebGPU) for cross-platform graphics
- **Procedural Generation**: Perlin noise-based terrain generation
- **Chunk System**: 16x16x64 block chunks for efficient memory usage
- **Serialization**: Binary world format using bincode and serde

## Troubleshooting

- **"cargo: command not found"**: Make sure Rust is installed and added to PATH
- **Build errors**: Ensure Visual Studio Build Tools are installed
- **Black screen**: Try updating your graphics drivers
- **Low FPS**: Run in release mode with `--release` flag

