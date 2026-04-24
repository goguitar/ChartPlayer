//! ChartPlayer - Rust port of ChartPlayer

pub mod camera;
pub mod scene;
pub mod midi;
pub mod audio;
pub mod song;
pub mod settings;
pub mod ui;

pub use camera::Camera3D;
pub use scene::{ChartScene3D, DrumPlayerScene3D, FretPlayerScene3D, KeysPlayerScene3D, PlayerScene3D, Scene3D, SceneVertex, SpriteFontDefinition, SpriteFontGlyph, SpriteLibrary, SpriteRegion};
pub use midi::{DrumVoice, DrumHit, DrumMidiDeviceConfiguration};
pub use audio::{AudioOutput, SharedSongPlayer, SongPlayer};
pub use song::{SongIndex, SongIndexEntry, SongData, SongInstrumentType};
pub use settings::SongPlayerSettings;
pub use ui::Player;

pub const VERSION: &str = "0.1.26";

pub const DEFAULT_VIEWPORT_WIDTH: i32 = 1024;
pub const DEFAULT_VIEWPORT_HEIGHT: i32 = 720;
pub const DEFAULT_SAMPLE_RATE: f64 = 48000.0;
pub const DEFAULT_NOTE_DISPLAY_SECONDS: f32 = 3.0;
pub const DEFAULT_NOTE_DISPLAY_DISTANCE: f32 = 600.0;
pub const DRUM_NOTE_DISPLAY_SECONDS: f32 = 3.0;
pub const DRUM_NOTE_DISPLAY_DISTANCE: f32 = 300.0;

pub fn init() {
    std::env::set_var("WAYLAND_DEBUG", "0");
}
