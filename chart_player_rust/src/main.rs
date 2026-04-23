//! ChartPlayer - A music chart player for Guitar Hero/Rock Band style games
use chart_player::{SongPlayer, Camera3D, ChartScene3D, SongPlayerSettings, constants::VERSION};

fn main() {
    env_logger::init();
    
    println!("ChartPlayer v{}", VERSION);
    println!("Copyright (c) 2024-2026 Mike Oliphant\n");
    
    let settings = SongPlayerSettings::new();
    let player = SongPlayer::new();
    let mut camera = Camera3D::new();
    let mut scene = ChartScene3D::new(Some(player));
    
    println!("Initialized with default settings.\n");
    println!("DEBUG: About to run_app...\n");
    
    run_app(&mut camera, &mut scene);
}

fn run_app(camera: &mut Camera3D, scene: &mut ChartScene3D) {
    use winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
    };
    
    println!("=== Starting GUI ===");
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let wayland = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
    let display = std::env::var("DISPLAY").unwrap_or_default();
    println!("XDG_SESSION_TYPE: {}", session);
    println!("WAYLAND_DISPLAY: {}", wayland);
    println!("DISPLAY: {}", display);
    
    println!("Creating EventLoop...");
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("ERROR: Failed to create event loop: {:?}", e);
            return;
        }
    };
    println!("EventLoop created OK");
    
    println!("Building window...");
    let window_result = WindowBuilder::new()
        .with_title("ChartPlayer v0.1.26 - Rust")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop);
    
    let window = match window_result {
        Ok(w) => w,
        Err(e) => {
            eprintln!("ERROR: Failed to create window: {:?}", e);
            return;
        }
    };
    println!("Window built OK");
    
    let size = window.inner_size();
    let id = window.id();
    println!("=== WINDOW READY ===");
    println!("Window ID: {:?}", id);
    println!("Size: {}x{}", size.width, size.height);
    println!("Title: ChartPlayer v0.1.26 - Rust");
    
    // Try to request attention to make window visible
    println!("Requesting window attention...");
    
    camera.viewport_width = size.width as i32;
    camera.viewport_height = size.height as i32;
    
    println!("ChartPlayer at {}x{}", camera.viewport_width, camera.viewport_height);
    println!("==================== WINDOW SHOULD BE VISIBLE NOW ====================");
    println!("Close window to exit");
    
    let _ = event_loop.run(move |event, _target| {
        match event {
            Event::WindowEvent { event, window_id } => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Close requested");
                    }
                    WindowEvent::Resized(size) => {
                        camera.viewport_width = size.width as i32;
                        camera.viewport_height = size.height as i32;
                        println!("Resized: {}x{}", size.width, size.height);
                    }
                    _ => {}
                }
            }
            Event::LoopExiting => {
                println!("Exiting");
            }
            _ => {}
        }
    });
}