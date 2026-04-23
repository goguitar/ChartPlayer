//! ChartPlayer - A music chart player for Guitar Hero/Rock Band style games
//! 
//! This is a Rust port of the original ChartPlayer C# project.
//! The library provides core functionality for playing and displaying music charts.

/// Initialize ChartPlayer - call once at application start
pub fn init() {
    // Enable debug logging for Wayland
    std::env::set_var("WAYLAND_DEBUG", "0");
}

pub mod camera;
pub mod scene;
pub mod midi;
pub mod audio;
pub mod song;
pub mod ui;
pub mod settings;

pub use camera::Camera3D;
pub use scene::{Scene3D, ChartScene3D, DrumPlayerScene3D, FretPlayerScene3D, KeysPlayerScene3D};
pub use midi::{DrumVoice, DrumHit, DrumMidiDeviceConfiguration};
pub use audio::SongPlayer;
pub use song::{SongIndex, SongIndexEntry, SongData, SongInstrumentType};
pub use ui::{SongPlayerInterface, LevelDisplay, TunerInterface};
pub use settings::SongPlayerSettings;

/// Application state and constants
pub mod constants {
    /// Version string
    pub const VERSION: &str = "0.1.26";
    
    /// Copyright notice
    pub const COPYRIGHT: &str = "Copyright (c) 2024-2026 Mike Oliphant";
    
    /// Default viewport dimensions
    pub const DEFAULT_VIEWPORT_WIDTH: i32 = 1024;
    pub const DEFAULT_VIEWPORT_HEIGHT: i32 = 720;
    
    /// Default playback sample rate
    pub const DEFAULT_SAMPLE_RATE: f64 = 48000.0;
    
    /// Note display settings
    pub const DEFAULT_NOTE_DISPLAY_SECONDS: f32 = 3.0;
    pub const DEFAULT_NOTE_DISPLAY_DISTANCE: f32 = 600.0;
    
    /// Drum note display settings
    pub const DRUM_NOTE_DISPLAY_SECONDS: f32 = 3.0;
    pub const DRUM_NOTE_DISPLAY_DISTANCE: f32 = 300.0;
}

/// Result type alias for ChartPlayer operations
pub type Result<T> = std::result::Result<T, ChartPlayerError>;

/// Error types for ChartPlayer
#[derive(Debug, thiserror::Error)]
pub enum ChartPlayerError {
    #[error("Audio error: {0}")]
    Audio(String),
    
    #[error("MIDI error: {0}")]
    Midi(String),
    
    #[error("Song loading error: {0}")]
    SongLoad(String),
    
    #[error("Graphics error: {0}")]
    Graphics(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}