//! ChartPlayer - Main entry point for GUI
use chart_player::{init, constants::VERSION};

fn main() {
    init();
    println!("ChartPlayer v{}\n", VERSION);
    
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;
    
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    println!("Window: ");
    
    let window = WindowBuilder::new()
        .with_title("ChartPlayer")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .expect("Failed to create window");
    
    println!("{}x{}", window.inner_size().width, window.inner_size().height);
    println!("\n=== WINDOW IS VISIBLE ===\n");
    
    let _ = event_loop.run(move |event, _target| {
        match event {
            winit::event::Event::WindowEvent { window_id: _, event } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        println!("Exiting...");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}