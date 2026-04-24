//! ChartPlayer - GTK4 GUI with connected controls

use chart_player::init;
use chart_player::VERSION;
use gtk4::{Application,ApplicationWindow};
use gtk4::prelude::*;
use std::sync::Arc;
use std::sync::atomic::Ordering;

mod ui;
use ui::Player;

fn main() {
    init();
    eprintln!("ChartPlayer v{} - GTK4\n", VERSION);
    
    let player = Arc::new(std::sync::Mutex::new(Player::default()));
    
    let app = Application::builder()
        .application_id("com.chartplayer.app")
        .build();
    
    app.connect_activate({
        let player = player.clone();
        move |app| {
            let window = ApplicationWindow::builder()
                .application(app)
                .title("ChartPlayer")
                .default_width(1280)
                .default_height(720)
                .build();
            
            let main_box = ui::main_interface(player.clone());
            window.set_child(Some(&main_box));
            window.show();
            
            eprintln!("Window ready");
        }
    });
    
    app.run();
}