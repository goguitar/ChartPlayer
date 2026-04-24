//! ChartPlayer - GTK4 GUI
use chart_player::{init, constants::VERSION};

fn main() {
    init();
    println!("ChartPlayer v{} - GTK4", VERSION);
    
    let app = gtk::Application::new("com.chartplayer.app", gtk::gio::ApplicationFlags::empty())
        .expect("GTK application failed");
    
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("ChartPlayer");
        window.set_default_size(1280, 720);
        
        let label = gtk::Label::new(Some("ChartPlayer v0.1.26\nWindow: 1280x720"));
        window.set_child(Some(&label));
        window.show();
    });
    
    app.run();
}