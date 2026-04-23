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
    
    let camera = Camera3D::new();
    let scene = ChartScene3D::new(Some(player));
    
    println!("\nInitialized with default settings.");
    println!("\nNote: Full GUI and audio playback requires additional platform-specific bindings.");
}