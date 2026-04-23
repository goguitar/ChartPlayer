//! ChartPlayer - A music chart player for Guitar Hero/Rock Band style games
//! 
//! This is a Rust port of the original ChartPlayer C# project.

use chart_player::{
    SongPlayer, Camera3D, ChartScene3D, FretPlayerScene3D, DrumPlayerScene3D,
    SongPlayerSettings, SongIndex, constants::VERSION,
};
use log::info;

fn main() {
    env_logger::init();
    
    info!("ChartPlayer v{} - A music chart player", VERSION);
    info!("Starting ChartPlayer...");
    
    println!("ChartPlayer v{}", VERSION);
    println!("Copyright (c) 2024-2026 Mike Oliphant");
    println!();
    println!("This is a Rust port of the original ChartPlayer project.");
    println!("Usage: chart_player <song_path>");
    
    let settings = SongPlayerSettings::new();
    let player = SongPlayer::new();
    
    let mut camera = Camera3D::new();
    let mut scene = ChartScene3D::new(Some(player));
    
    println!("\nInitialized with default settings.");
    
    run_app(&mut camera, &mut scene);
}

fn run_app(camera: &mut Camera3D, scene: &mut ChartScene3D) {
    use winit::{
        event::{Event, WindowEvent, KeyEvent, MouseButton},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    
    println!("\nStarting GUI...");
    
    let event_loop = EventLoop::new().unwrap();
    
    let window = match WindowBuilder::new()
        .with_title("ChartPlayer")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .with_resizable(true)
        .build(&event_loop) 
    {
        Ok(window) => window,
        Err(e) => {
            eprintln!("Failed to create window: {:?}", e);
            return;
        }
    };
    
    println!("Window created: {}x{}", window.inner_size().width, window.inner_size().height);
    
    camera.viewport_width = window.inner_size().width as i32;
    camera.viewport_height = window.inner_size().height as i32;
    
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { window_id: _, event } => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Close requested");
                        target.exit();
                    }
                    WindowEvent::Resized(size) => {
                        camera.viewport_width = size.width as i32;
                        camera.viewport_height = size.height as i32;
                        println!("Resized: {}x{}", size.width, size.height);
                    }
                    WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                        info!("Key: pressed:{}", event.state.is_pressed());
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        info!("Modifiers: {:?}", modifiers);
                    }
                    WindowEvent::CursorMoved { device_id: _, position, .. } => {
                        info!("Cursor: {:?}", position);
                    }
                    WindowEvent::MouseInput { device_id: _, button, state, .. } => {
                        info!("Mouse button: {:?} state: {:?}", button, state);
                    }
                    WindowEvent::Focused(focused) => {
                        info!("Focused: {}", focused);
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent { event, .. } => {
                info!("Device event: {:?}", event);
            }
            _ => {}
        }
        
        target.set_control_flow(ControlFlow::Wait);
    });
    
    println!("GUI shutdown complete");
}