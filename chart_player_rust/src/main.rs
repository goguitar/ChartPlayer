use chart_player::VERSION;
use chart_player::init;
use gtk4::{Application, ApplicationWindow};
use gtk4::prelude::*;

mod ui;

fn main() {
    init();
    eprintln!("ChartPlayer v{} - GTK4 GUI\n", VERSION);
    
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
        
        let main_box = ui::main_interface();
        window.set_child(Some(&main_box));
        window.show();
    });
    
    app.run();
}