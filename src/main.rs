mod block;
mod camera;
mod chunk;
mod input;
mod mesh;
mod physics;
mod raycast;
mod renderer;
mod vertex;
mod world;
mod world_gen;

#[cfg(test)]
mod tests;

use camera::Camera;
use input::InputHandler;
use physics::Player;
use renderer::Renderer;
use std::sync::Arc;
use std::time::Instant;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use world::World;
use world_gen::WorldGenerator;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Rustcraft - Voxel Game")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);

    // Grab and hide the cursor for FPS-style controls
    window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
        .unwrap_or_else(|e| eprintln!("Failed to grab cursor: {}", e));
    window.set_cursor_visible(false);

    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    let aspect = renderer.size.width as f32 / renderer.size.height as f32;
    let mut camera = Camera::new(aspect);
    let mut player = Player::new(camera.position);
    let mut input_handler = InputHandler::new();
    let mut world_needs_update = false;

    let world_path = "world.dat";
    let mut world = World::load(world_path).unwrap_or_else(|_| {
        println!("Creating new world...");
        World::new(12345)
    });

    let generator = WorldGenerator::new(world.seed);

    // Generate initial chunks around spawn
    for x in -3..=3 {
        for z in -3..=3 {
            world.load_or_generate_chunk(x, z, &generator);
        }
    }

    renderer.update_mesh(&world, &camera);

    let mut last_frame = Instant::now();
    let mut frame_count = 0;
    let mut last_fps_update = Instant::now();

    event_loop.set_control_flow(ControlFlow::Poll);

    let _ = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                println!("Saving world...");
                if let Err(e) = world.save(world_path) {
                    eprintln!("Failed to save world: {}", e);
                } else {
                    println!("World saved successfully!");
                }
                elwt.exit();
            }
            WindowEvent::Resized(physical_size) => {
                renderer.resize(*physical_size);
                camera.update_aspect(physical_size.width as f32 / physical_size.height as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                input_handler.process_keyboard(event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                input_handler.process_mouse_button(*state, *button);
                
                // Handle block interactions on mouse click
                if *state == ElementState::Pressed
                    && input_handler.handle_block_interaction(&camera, &mut world) {
                    world_needs_update = true;
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;

                // Update camera look direction
                input_handler.update_camera(&mut camera);

                // Update player physics and movement
                input_handler.update_player(&mut player, &camera, delta_time);
                player.apply_physics(delta_time, &world);

                // Sync camera position with player
                camera.position = player.position + glam::Vec3::new(0.0, 1.6, 0.0); // Eye height

                // Load chunks around camera
                let cam_chunk_x = (camera.position.x / 16.0).floor() as i32;
                let cam_chunk_z = (camera.position.z / 16.0).floor() as i32;

                for dx in -3..=3 {
                    for dz in -3..=3 {
                        world.load_or_generate_chunk(cam_chunk_x + dx, cam_chunk_z + dz, &generator);
                    }
                }

                // Update mesh if world changed
                if world_needs_update {
                    renderer.update_mesh(&world, &camera);
                    world_needs_update = false;
                } else {
                    renderer.update_mesh(&world, &camera);
                }
                
                renderer.update_camera(&camera);

                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("{:?}", e),
                }

                frame_count += 1;
                if now.duration_since(last_fps_update).as_secs() >= 1 {
                    println!(
                        "FPS: {} | Pos: ({:.1}, {:.1}, {:.1}) | Vel: ({:.1}, {:.1}, {:.1}) | Ground: {}",
                        frame_count, player.position.x, player.position.y, player.position.z,
                        player.velocity.x, player.velocity.y, player.velocity.z,
                        player.on_ground
                    );
                    frame_count = 0;
                    last_fps_update = now;
                }
            }
            _ => {}
        },
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            input_handler.process_mouse_motion(delta);
        }
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => {}
    });
}

