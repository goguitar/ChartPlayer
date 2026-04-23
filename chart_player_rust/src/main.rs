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
    
    run_app(&mut camera, &mut scene);
}

fn run_app(camera: &mut Camera3D, scene: &mut ChartScene3D) {
    use winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    
    println!("Starting GUI...");
    println!("Session: {}", std::env::var("XDG_SESSION_TYPE").unwrap_or_default());
    
    let event_loop = EventLoop::new().unwrap();
    
    println!("Creating window...");
    
    let window = match WindowBuilder::new()
        .with_title("ChartPlayer v0.1.26")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop) 
    {
        Ok(window) => window,
        Err(e) => {
            eprintln!("Failed: {:?}", e);
            return;
        }
    };
    
    let size = window.inner_size();
    println!("Window: {}x{}", size.width, size.height);
    camera.viewport_width = size.width as i32;
    camera.viewport_height = size.height as i32;
    
    println!("ChartPlayer running - close window to exit");
    
    event_loop.run(move |event, _target| {
        match event {
            Event::WindowEvent { event, .. } => {
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
                println!("Loop exiting");
            }
            _ => {}
        }
    });
    
    println!("GUI exited");
}