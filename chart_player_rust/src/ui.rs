//! UI module for interface components
//! 
//! Provides UI elements for song display, settings, and level meters.

use crate::audio::SongPlayer;
use crate::midi::NoteDetector;
use crate::song::{SongIndex, SongIndexEntry, SongTuningMode, SongInstrumentType};
use crate::scene::{FretPlayerScene3D, ChartScene3D};
use crate::settings::SongPlayerSettings;
use std::time::{Duration, SystemTime};

/// Audio level display
#[derive(Debug, Clone)]
pub struct LevelDisplay {
    pub warn_level: f64,
    last_value: f64,
    clip: f64,
}

impl LevelDisplay {
    pub fn new() -> Self {
        Self {
            warn_level: 0.8,
            last_value: 0.0,
            clip: 0.0,
        }
    }
    
    pub fn set_value(&mut self, value: f64) {
        if value < 0.01 {
            self.clip -= 0.1;
            if self.clip <= 0.0 {
                self.clip = 0.0;
            }
        }
        
        if value >= 1.0 {
            self.clip = 1.0;
        } else if value >= self.warn_level {
            self.clip = 1.0;
        }
        
        self.last_value = value;
    }
}

impl Default for LevelDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Tuner interface for guitar tuning display
#[derive(Debug, Clone)]
pub struct TunerInterface {
    pub current_pitch: f32,
    pub closest_note: i32,
    pub cents_offset: f32,
    target_notes: Vec<(&'static str, i32, f32)>,
    pitch_history: Vec<Option<f32>>,
    queue_size: usize,
    frame_count: i32,
    start_time: Option<SystemTime>,
    last_closest_note: i32,
    last_cents_offset: Option<f32>,
    running_cents_offset: f32,
}

impl TunerInterface {
    pub fn new() -> Self {
        let note_names = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];
        let mut target_notes = Vec::new();
        
        for octave in 0..7 {
            for (i, name) in note_names.iter().enumerate() {
                let midi = (octave + 1) * 12 + i as i32;
                let freq = get_midi_note_frequency(midi);
                target_notes.push((*name, octave, freq));
            }
        }
        
        Self {
            current_pitch: 0.0,
            closest_note: 9, // A
            cents_offset: 0.0,
            target_notes,
            pitch_history: Vec::new(),
            queue_size: 40,
            frame_count: 0,
            start_time: None,
            last_closest_note: -1,
            last_cents_offset: None,
            running_cents_offset: 0.0,
        }
    }
    
    pub fn update(&mut self, pitch: f32) {
        self.current_pitch = pitch;
        
        if self.frame_count > 0 {
            self.frame_count -= 1;
        } else {
            self.frame_count = 10;
        }
        
        if pitch > 20.0 {
            let mut min_diff = f32::MAX;
            
            for (name, octave, freq) in &self.target_notes {
                let diff = (freq - pitch).abs();
                if diff < min_diff {
                    min_diff = diff;
                    self.closest_note = name_to_note(name) + octave * 12;
                    self.cents_offset = (1200.0 * (pitch / freq).log2()) as f32;
                } else {
                    break;
                }
            }
            
            if let Some(last) = self.last_cents_offset {
                self.cents_offset = 0.1 * self.cents_offset + 0.9 * last;
            }
            self.last_cents_offset = Some(self.cents_offset);
            
            self.pitch_history.push(Some(self.cents_offset));
            
            if (self.cents_offset - self.running_cents_offset).abs() > 10.0 {
                self.running_cents_offset = self.cents_offset;
            } else {
                self.running_cents_offset = 0.99 * self.running_cents_offset + 0.01 * self.cents_offset;
            }
        } else {
            self.pitch_history.push(None);
            self.last_cents_offset = None;
        }
        
        while self.pitch_history.len() > self.queue_size {
            self.pitch_history.remove(0);
        }
        
        if self.last_closest_note != self.closest_note {
            self.last_closest_note = self.closest_note;
        }
    }
}

/// Help dialog
#[derive(Debug, Clone)]
pub struct HelpDialog {
    pub title: String,
    pub content: String,
}

impl HelpDialog {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
        }
    }
}

/// Song player interface
#[derive(Debug, Clone)]
pub struct SongPlayerInterface {
    pub loop_marker_start: f32,
    pub loop_marker_end: f32,
    
    song_index: Option<SongIndex>,
    song_player: Option<SongPlayer>,
    current_instrument: SongInstrumentType,
    settings: SongPlayerSettings,
    need_seek: bool,
    seek_secs: f32,
}

impl SongPlayerInterface {
    pub fn new(settings: SongPlayerSettings) -> Self {
        Self {
            loop_marker_start: -1.0,
            loop_marker_end: -1.0,
            song_index: None,
            song_player: None,
            current_instrument: SongInstrumentType::LeadGuitar,
            settings,
            need_seek: false,
            seek_secs: 0.0,
        }
    }
    
    pub fn set_song_index(&mut self, index: SongIndex) {
        self.song_index = Some(index);
    }
    
    pub fn set_song(&mut self, entry: &SongIndexEntry, instrument: SongInstrumentType) {
        self.current_instrument = instrument;
    }
    
    pub fn toggle_loop_start(&mut self, second: f32) {
        if self.loop_marker_start == -1.0 {
            if self.loop_marker_end == -1.0 || self.loop_marker_end > second {
                self.loop_marker_start = second;
            }
        } else {
            self.loop_marker_start = -1.0;
        }
    }
    
    pub fn toggle_loop_end(&mut self, second: f32) {
        if self.loop_marker_end == -1.0 {
            if self.loop_marker_start == -1.0 || self.loop_marker_start < second {
                self.loop_marker_end = second;
            }
        } else {
            self.loop_marker_end = -1.0;
        }
    }
    
    pub fn seek_time(&mut self, secs: f32) {
        self.seek_secs = secs;
        self.need_seek = true;
    }
    
    pub fn check_loop_markers(&mut self) {
        if let Some(ref player) = self.song_player {
            if self.loop_marker_start != -1.0 && self.loop_marker_end != -1.0 {
                if player.current_second > self.loop_marker_end {
                    self.seek_time(self.loop_marker_start);
                }
            }
        }
    }
}

// Helper functions
fn get_midi_note_frequency(midi_note: i32) -> f32 {
    let a4_midi = 57;
    let a4_freq = 440.0;
    let half_step_ratio = 2.0_f32.powf(1.0 / 12.0);
    a4_freq / half_step_ratio.powi(a4_midi - midi_note)
}

fn name_to_note(name: &str) -> i32 {
    match name {
        "C" => 0, "C#" => 1, "D" => 2, "Eb" => 3, "E" => 4, "F" => 5,
        "F#" => 6, "G" => 7, "Ab" => 8, "A" => 9, "Bb" => 10, "B" => 11,
        _ => 0,
    }
}