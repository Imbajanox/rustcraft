This will be a Minecraft like Voxel game made in Rust

# Rustcraft - Voxel Game Prototype

A Minecraft-like voxel game prototype built in Rust with procedural world generation, multiple block types, and first-person controls.

## Features

- **7 Block Types**: Dirt, Sand, Grass, Wood, Leaves, Planks, and Glass
- **Procedural World Generation**: Infinite terrain with hills, valleys, and trees
- **World Saving**: Automatically saves world state when exiting
- **First-Person Camera**: WASD movement and mouse look controls
- **Chunk-based Rendering**: Efficient rendering with culling of hidden faces

## Controls

- **W/A/S/D**: Move forward/left/backward/right
- **Space**: Move up
- **Left Shift**: Move down
- **Left Mouse Button**: Hold and move mouse to look around
- **Escape**: Save and quit

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
- Performance is better in release mode (with `--release` flag)
- First run may take longer as it generates the initial world chunks
- Console will show FPS and current position
- The world will be saved automatically when you press Escape or close the window

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

