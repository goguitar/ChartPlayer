//! Scene module for 3D rendering
//! 
//! Provides various 3D scene implementations for different instrument types
//! including drums, guitar fretboard, and keyboard displays.

use crate::camera::Camera3D;
use crate::audio::SongPlayer;
use crate::midi::MidiHandler;
use crate::song::{SongData, SongBeat, SongNote, SongDrumNote, SongKeyboardNote, SongStructure};
use crate::constants::*;
use cgmath::{Vector3, Matrix4, Point3};
use std::collections::HashMap;

/// Base 3D scene with common rendering functionality
#[derive(Debug, Clone)]
pub struct Scene3D {
    pub camera: Camera3D,
    
    pub fog_enabled: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 4],
}

impl Scene3D {
    pub fn new() -> Self {
        let mut camera = Camera3D::new();
        camera.position = Vector3::new(0.0, 0.0, 5.0);
        camera.forward = Vector3::new(0.0, 0.0, -1.0);
        
        Self {
            camera,
            fog_enabled: false,
            fog_start: 400.0,
            fog_end: 700.0,
            fog_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
    
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.camera.get_projection_matrix()
    }
    
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        self.camera.get_view_matrix()
    }
}

impl Default for Scene3D {
    fn default() -> Self {
        Self::new()
    }
}

/// Base chart scene with common beat/note display functionality
#[derive(Debug, Clone)]
pub struct ChartScene3D {
    pub base: Scene3D,
    
    pub note_display_seconds: f32,
    pub note_display_distance: f32,
    pub num_notes_detected: i32,
    pub num_notes_total: i32,
    pub current_bpm: f32,
    pub lefty_mode: bool,
    pub current_time_offset: f32,
    
    player: Option<SongPlayer>,
    time_scale: f32,
    current_time: f32,
    start_time: f32,
    end_time: f32,
    highway_start_x: f32,
    highway_end_x: f32,
    score_start_secs: f32,
    start_beat_position: i32,
    white_half_alpha: [f32; 4],
    white_three_quarters_alpha: [f32; 4],
}

impl ChartScene3D {
    pub fn new(player: Option<SongPlayer>) -> Self {
        Self {
            base: Scene3D::new(),
            note_display_seconds: DEFAULT_NOTE_DISPLAY_SECONDS,
            note_display_distance: DEFAULT_NOTE_DISPLAY_DISTANCE,
            num_notes_detected: 0,
            num_notes_total: 0,
            current_bpm: 0.0,
            lefty_mode: false,
            current_time_offset: 0.0,
            player,
            time_scale: 0.0,
            current_time: 0.0,
            start_time: 0.0,
            end_time: 0.0,
            highway_start_x: 0.0,
            highway_end_x: 0.0,
            score_start_secs: 0.0,
            start_beat_position: 0,
            white_half_alpha: [1.0, 1.0, 1.0, 0.5],
            white_three_quarters_alpha: [1.0, 1.0, 1.0, 0.75],
        }
    }
    
    pub fn set_player(&mut self, player: SongPlayer) {
        self.player = Some(player);
    }
    
    pub fn reset_score(&mut self, score_start_secs: f32) {
        self.score_start_secs = score_start_secs;
        self.num_notes_detected = 0;
        self.num_notes_total = 0;
    }
    
    pub fn update(&mut self, current_time: f32) {
        self.time_scale = self.note_display_distance / self.note_display_seconds;
        self.current_time = current_time;
        self.start_time = current_time;
        self.end_time = current_time + self.note_display_seconds;
        
        self.base.camera.mirror_left_right = self.lefty_mode;
        self.update_camera();
    }
    
    pub fn update_camera(&mut self) {
        // Override in subclasses
    }
    
    /// Gets the start note index for visible notes
    pub fn get_start_note<T: AsRef<dyn SongEventTrait>>(
        &self, 
        time_offset: f32, 
        min_length: f32, 
        start_note_position: i32, 
        notes: &[T]
    ) -> i32 {
        let mut position = start_note_position.max(0) as usize;
        
        while position > 0 {
            let note = notes[position].as_ref();
            let end_time = note.end_time().max(note.time_offset() + min_length);
            
            if end_time < time_offset {
                break;
            }
            position -= 1;
        }
        
        while position < notes.len() {
            let note = notes[position].as_ref();
            let end_time = note.end_time().max(note.time_offset() + min_length);
            
            if end_time > time_offset {
                break;
            }
            position += 1;
        }
        
        position as i32
    }
    
    /// Gets the end note index for visible notes
    pub fn get_end_note<T: AsRef<dyn SongEventTrait>>(
        &self,
        start_position: i32,
        end_time: f32,
        notes: &[T]
    ) -> i32 {
        let mut position = start_position as usize;
        
        while position < notes.len() {
            let note = notes[position].as_ref();
            if note.time_offset() > end_time {
                break;
            }
            position += 1;
        }
        
        if position == notes.len() {
            position -= 1;
        }
        
        position as i32
    }
}

impl Default for ChartScene3D {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Trait for song events (notes, beats)
pub trait SongEventTrait {
    fn time_offset(&self) -> f32;
    fn end_time(&self) -> f32;
}

impl SongEventTrait for SongBeat {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.time_offset
    }
}

impl SongEventTrait for SongNote {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.end_time
    }
}

impl SongEventTrait for SongDrumNote {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.end_time
    }
}

impl SongEventTrait for SongKeyboardNote {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.end_time
    }
}

/// Drum player scene for displaying drum notes
#[derive(Debug, Clone)]
pub struct DrumPlayerScene3D {
    pub base: ChartScene3D,
    
    num_lanes: i32,
    camera_distance: f32,
    position_lane: f32,
    notes_detected: Vec<Option<f32>>,
    start_note_position: i32,
    detection_tolerance_secs: f32,
    
    drum_hits: Vec<DrumHit>,
    drum_hit_index: usize,
}

impl DrumPlayerScene3D {
    pub fn new(player: SongPlayer) -> Self {
        let num_notes = player.get_drum_notes().len();
        
        Self {
            base: ChartScene3D::new(Some(player)),
            num_lanes: 5,
            camera_distance: 75.0,
            position_lane: 2.5,
            notes_detected: vec![None; num_notes],
            start_note_position: 0,
            detection_tolerance_secs: 0.1,
            drum_hits: Vec::new(),
            drum_hit_index: 0,
        }
    }
    
    pub fn handle_drum_hit(&mut self, hit: DrumHit) {
        self.drum_hits.push(hit);
    }
    
    pub fn get_lane_position(&self, lane: f32) -> f32 {
        lane * 15.0
    }
    
    pub fn reset_score(&mut self, score_start_secs: f32) {
        for note in &mut self.notes_detected {
            *note = None;
        }
        self.base.reset_score(score_start_secs);
    }
}

impl MidiHandler for DrumPlayerScene3D {
    fn handle_note_on(&mut self, channel: i32, note_number: i32, velocity: f32, sample_offset: i32) {
        let hit = DrumHit {
            channel,
            note_number,
            velocity,
            sample_offset,
            is_live: true,
            dimension_value: 0.0,
        };
        self.handle_drum_hit(hit);
    }
    
    fn handle_poly_pressure(&mut self, channel: i32, note_number: i32, pressure: f32, sample_offset: i32) {
        let hit = DrumHit {
            channel,
            note_number,
            velocity: pressure,
            sample_offset,
            is_live: true,
            dimension_value: 0.0,
        };
        self.handle_drum_hit(hit);
    }
}

impl Default for DrumPlayerScene3D {
    fn default() -> Self {
        Self::new(SongPlayer::new())
    }
}

/// Drum hit information
#[derive(Debug, Clone, Copy)]
pub struct DrumHit {
    pub channel: i32,
    pub note_number: i32,
    pub velocity: f32,
    pub sample_offset: i32,
    pub is_live: bool,
    pub dimension_value: f32,
}

/// Guitar fret player scene
#[derive(Debug, Clone)]
pub struct FretPlayerScene3D {
    pub base: ChartScene3D,
    
    pub display_notes: bool,
    pub detect_semitone_offset: f32,
    pub capo_fret: i32,
    
    num_frets: i32,
    min_fret: f32,
    max_fret: f32,
    notes_detected: Vec<i8>,
    start_note_position: i32,
    string_colors: [[f32; 4]; 6],
    string_color_names: Vec<String>,
    string_color_offset: i32,
    target_focus_fret: f32,
}

impl FretPlayerScene3D {
    pub fn new(player: SongPlayer) -> Self {
        let instrument_notes_len = player.get_instrument_notes().len();
        Self {
            base: ChartScene3D::new(Some(player)),
            display_notes: true,
            detect_semitone_offset: 0.0,
            capo_fret: 0,
            num_frets: 24,
            min_fret: 0.0,
            max_fret: 4.0,
            notes_detected: vec![0; instrument_notes_len],
            start_note_position: 0,
            string_colors: [
                [0.1, 0.8, 0.0, 1.0],  // Green
                [1.0, 0.0, 0.0, 1.0],    // Red
                [1.0, 1.0, 0.0, 1.0],    // Yellow
                [0.0, 0.6, 1.0, 1.0],    // Cyan
                [1.0, 0.5, 0.0, 1.0],    // Orange
                [0.1, 0.8, 0.0, 1.0],    // Green (7th string)
            ],
            string_color_names: vec![
                "Green".to_string(),
                "Red".to_string(),
                "Yellow".to_string(),
                "Cyan".to_string(),
                "Orange".to_string(),
                "Purple".to_string(),
            ],
            string_color_offset: 1,
            target_focus_fret: 2.0,
        }
    }
    
    pub fn get_string_height(&self, string: f32) -> f32 {
        3.0 + (string * 4.0)
    }
    
    pub fn reset_score(&mut self, score_start_secs: f32) {
        for note in &mut self.notes_detected {
            *note = 0;
        }
        self.base.reset_score(score_start_secs);
    }
}

impl Default for FretPlayerScene3D {
    fn default() -> Self {
        Self::new(SongPlayer::new())
    }
}

/// Keyboard player scene
#[derive(Debug, Clone)]
pub struct KeysPlayerScene3D {
    pub base: ChartScene3D,
    
    min_key: i32,
    max_key: i32,
    target_camera_distance: f32,
    camera_distance: f32,
    position_key: f32,
    start_note_position: i32,
    
    scale_white_black: [i32; 12],
    scale_offsets: [f32; 12],
}

impl KeysPlayerScene3D {
    pub fn new(player: SongPlayer) -> Self {
        Self {
            base: ChartScene3D::new(Some(player)),
            min_key: 48,
            max_key: 72,
            target_camera_distance: 64.0,
            camera_distance: 70.0,
            position_key: 60.0,
            start_note_position: 0,
            scale_white_black: [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0],
            scale_offsets: [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0],
        }
    }
    
    pub fn get_key_position(&self, key: f32) -> f32 {
        let int_key = key as i32;
        
        if key == int_key as f32 {
            let octave = (int_key - self.min_key) / 12;
            let note_in_octave = (int_key - self.min_key) % 12;
            (self.scale_offsets[note_in_octave as usize] + (octave as f32 * 7.0)) * 8.0
        } else {
            let pos = self.get_key_position(int_key as f32);
            let frac = key - int_key as f32;
            pos + (frac * 8.0)
        }
    }
}

impl Default for KeysPlayerScene3D {
    fn default() -> Self {
        Self::new(SongPlayer::new())
    }
}