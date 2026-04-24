//! ChartPlayer - Main entry point for GUI
use chart_player::init;
use chart_player::VERSION;

fn main() {
    init();
    println!("ChartPlayer v{}\n", VERSION);
    
    use winit::event_loop::EventLoop;
    use winit::window::Window;
    use winit::dpi::LogicalSize;
    
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    
    let window = event_loop.create_window(Window::default_attributes()
        .with_title("CHARTPLAYER")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
    ).expect("Failed to create window");
    
    let size = window.inner_size();
    println!("Window: {}x{}", size.width, size.height);
    println!("Running - close window to exit\n");
    
    let _ = event_loop.run(move |event, _target| {
        match event {
            winit::event::Event::WindowEvent { window_id: _, event } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        println!("Exit");
                    }
                    winit::event::WindowEvent::Resized(s) => {
                        println!("Resize: {}x{}", s.width, s.height);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}