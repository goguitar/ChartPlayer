//! Settings module for player configuration

use crate::song::{SongInstrumentType, SongTuningMode};

/// Song player settings
#[derive(Debug, Clone)]
pub struct SongPlayerSettings {
    pub song_path: Option<String>,
    pub invert_strings: bool,
    pub lefty_mode: bool,
    pub mute_part_stems: bool,
    pub song_tuning_mode: SongTuningMode,
    pub bass_using_guitar: bool,
    pub note_display_seconds: f32,
    pub drums_note_display_seconds: f32,
    pub keys_note_display_seconds: f32,
    pub current_instrument: SongInstrumentType,
    pub song_list_sort_column: Option<String>,
    pub song_list_sort_reversed: bool,
    pub ui_scale: f32,
    pub drum_midi_map_name: Option<String>,
}

impl SongPlayerSettings {
    pub fn new() -> Self {
        Self {
            song_path: None,
            invert_strings: false,
            lefty_mode: false,
            mute_part_stems: false,
            song_tuning_mode: SongTuningMode::A440,
            bass_using_guitar: false,
            note_display_seconds: 3.0,
            drums_note_display_seconds: 3.0,
            keys_note_display_seconds: 3.0,
            current_instrument: SongInstrumentType::LeadGuitar,
            song_list_sort_column: None,
            song_list_sort_reversed: false,
            ui_scale: 1.0,
            drum_midi_map_name: None,
        }
    }
}

impl Default for SongPlayerSettings {
    fn default() -> Self {
        Self::new()
    }
}