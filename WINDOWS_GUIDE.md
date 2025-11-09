# Windows Build and Run Instructions

## Step-by-Step Guide to Run Rustcraft on Windows

### 1. Install Prerequisites

#### Install Rust
1. Go to [https://rustup.rs/](https://rustup.rs/)
2. Download `rustup-init.exe`
3. Run the installer
4. Follow the on-screen instructions (accept defaults)
5. Restart your terminal/command prompt after installation

To verify installation, open a new terminal and run:
```cmd
rustc --version
cargo --version
```

#### Install Visual Studio Build Tools
1. Go to [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/)
2. Download "Build Tools for Visual Studio"
3. Run the installer
4. Select "Desktop development with C++"
5. Click Install (this may take a while)

### 2. Download/Clone the Project

If you have Git installed:
```cmd
git clone https://github.com/Imbajanox/rustcraft.git
cd rustcraft
```

Or download the repository as a ZIP file and extract it.

### 3. Build the Game

Open Command Prompt or PowerShell and navigate to the project directory:

```cmd
cd path\to\rustcraft
```

Build in release mode (recommended for better performance):
```cmd
cargo build --release
```

This will take 5-10 minutes the first time as it downloads and compiles all dependencies.

### 4. Run the Game

After building, run the game:
```cmd
cargo run --release
```

Or run the executable directly:
```cmd
target\release\rustcraft.exe
```

### 5. Game Controls

- **W** - Move forward
- **A** - Move left
- **S** - Move backward
- **D** - Move right
- **Space** - Move up
- **Left Shift** - Move down
- **Left Mouse Button (hold)** - Look around with mouse
- **Escape** - Save world and quit

### 6. Gameplay

- The game generates an infinite procedural world
- Walk around and explore the terrain
- Trees, grass, and terrain are randomly generated
- Your world is automatically saved to `world.dat` when you quit
- Next time you run the game, it will load your saved world

### Troubleshooting

#### "cargo is not recognized as an internal or external command"
- Make sure Rust is installed correctly
- Restart your terminal after installation
- Check that Rust is in your PATH environment variable

#### Build Errors
- Ensure Visual Studio Build Tools are installed with C++ support
- Try running the terminal as Administrator
- Make sure you have a stable internet connection for downloading dependencies

#### Black Screen or Crashes
- Update your graphics drivers
- Make sure your GPU supports DirectX 11 or higher
- Try running in compatibility mode

#### Low FPS
- Make sure you're building and running in release mode with `--release`
- Close other applications
- Lower the render distance (requires code modification)

#### World Data
- Your world is saved in `world.dat` in the project directory
- To start fresh, delete `world.dat`
- The world seed is hardcoded to `12345` (can be changed in code)

### Performance Tips

1. **Always use release mode**: `cargo run --release` is much faster than debug mode
2. **First run is slower**: Initial chunk generation takes time
3. **Console output**: The console shows FPS and your position

### Additional Information

- The game uses **wgpu** for cross-platform graphics rendering
- Chunks are **16x16x64** blocks
- Render distance is set to **3 chunks** in each direction
- Block types: Dirt, Sand, Grass, Wood, Leaves, Planks, Glass

### Development

To modify the game:
1. Edit source files in the `src/` directory
2. Rebuild with `cargo build --release`
3. Run tests with `cargo test`
4. Check code quality with `cargo clippy`

Enjoy exploring the voxel world!
