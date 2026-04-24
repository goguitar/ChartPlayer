//! ChartPlayer - GTK4 GUI
use chart_player::{init, constants::VERSION};
use gtk4::{Application, ApplicationWindow, Label};
use gtk4::prelude::*;

fn main() {
    init();
    eprintln!("ChartPlayer v{} - GTK4\n", VERSION);
    
    let app = Application::builder()
        .application_id("com.chartplayer.app")
        .build();
    
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("ChartPlayer")
            .default_width(1280)
            .default_height(720)
            .build();
        
        let label = Label::builder()
            .label("ChartPlayer v0.1.26\n1280x720")
            .build();
        
        window.set_child(Some(&label));
        window.show();
    });
    
    app.run();
}