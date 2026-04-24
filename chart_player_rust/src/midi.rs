//! MIDI handling module
//!
//! Provides MIDI input processing, drum mapping, and note detection
//! for various MIDI controllers.

use std::collections::HashMap;
use std::fmt;
use std::sync::{OnceLock, RwLock};

/// Drum kit piece enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumKitPiece {
    None,
    Kick,
    Snare,
    HiHat,
    Crash,
    Crash2,
    Crash3,
    Ride,
    Ride2,
    Tom1,
    Tom2,
    Tom3,
    Tom4,
    Tom5,
    Flexi1,
    Flexi2,
    Flexi3,
    Flexi4,
}

impl fmt::Display for DrumKitPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrumKitPiece::None => write!(f, "None"),
            DrumKitPiece::Kick => write!(f, "Kick"),
            DrumKitPiece::Snare => write!(f, "Snare"),
            DrumKitPiece::HiHat => write!(f, "HiHat"),
            DrumKitPiece::Crash => write!(f, "Crash"),
            DrumKitPiece::Crash2 => write!(f, "Crash2"),
            DrumKitPiece::Crash3 => write!(f, "Crash3"),
            DrumKitPiece::Ride => write!(f, "Ride"),
            DrumKitPiece::Ride2 => write!(f, "Ride2"),
            DrumKitPiece::Tom1 => write!(f, "Tom1"),
            DrumKitPiece::Tom2 => write!(f, "Tom2"),
            DrumKitPiece::Tom3 => write!(f, "Tom3"),
            DrumKitPiece::Tom4 => write!(f, "Tom4"),
            DrumKitPiece::Tom5 => write!(f, "Tom5"),
            DrumKitPiece::Flexi1 => write!(f, "Flexi1"),
            DrumKitPiece::Flexi2 => write!(f, "Flexi2"),
            DrumKitPiece::Flexi3 => write!(f, "Flexi3"),
            DrumKitPiece::Flexi4 => write!(f, "Flexi4"),
        }
    }
}

/// Drum kit piece type for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumKitPieceType {
    None,
    Kick,
    Snare,
    HiHat,
    Crash,
    Ride,
    Tom,
    Flexi,
}

/// Drum articulation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumArticulation {
    None,
    DrumHead,
    DrumHeadEdge,
    DrumRim,
    SideStick,
    HiHatClosed,
    HiHatOpen,
    HiHatChick,
    HiHatSplash,
    CymbalEdge,
    CymbalBow,
    CymbalBell,
    CymbalChoke,
    FlexiA,
    FlexiB,
    FlexiC,
}

impl fmt::Display for DrumArticulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrumArticulation::None => write!(f, "None"),
            DrumArticulation::DrumHead => write!(f, "Head"),
            DrumArticulation::DrumHeadEdge => write!(f, "HeadEdge"),
            DrumArticulation::DrumRim => write!(f, "Rim"),
            DrumArticulation::SideStick => write!(f, "SideStick"),
            DrumArticulation::HiHatClosed => write!(f, "Closed"),
            DrumArticulation::HiHatOpen => write!(f, "Open"),
            DrumArticulation::HiHatChick => write!(f, "Chick"),
            DrumArticulation::HiHatSplash => write!(f, "Splash"),
            DrumArticulation::CymbalEdge => write!(f, "Edge"),
            DrumArticulation::CymbalBow => write!(f, "Bow"),
            DrumArticulation::CymbalBell => write!(f, "Bell"),
            DrumArticulation::CymbalChoke => write!(f, "Choke"),
            DrumArticulation::FlexiA => write!(f, "FlexiA"),
            DrumArticulation::FlexiB => write!(f, "FlexiB"),
            DrumArticulation::FlexiC => write!(f, "FlexiC"),
        }
    }
}

/// Represents a drum voice (combination of piece and articulation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrumVoice {
    pub kit_piece: DrumKitPiece,
    pub articulation: DrumArticulation,
}

impl fmt::Display for DrumVoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{:?}", self.kit_piece, self.articulation)
    }
}

impl DrumVoice {
    pub fn new(kit_piece: DrumKitPiece, articulation: DrumArticulation) -> Self {
        Self {
            kit_piece,
            articulation,
        }
    }

    pub fn get_kit_piece_type(&self) -> DrumKitPieceType {
        match self.kit_piece {
            DrumKitPiece::None => DrumKitPieceType::None,
            DrumKitPiece::Kick => DrumKitPieceType::Kick,
            DrumKitPiece::Snare => DrumKitPieceType::Snare,
            DrumKitPiece::HiHat => DrumKitPieceType::HiHat,
            DrumKitPiece::Crash | DrumKitPiece::Crash2 | DrumKitPiece::Crash3 => {
                DrumKitPieceType::Crash
            }
            DrumKitPiece::Ride | DrumKitPiece::Ride2 => DrumKitPieceType::Ride,
            DrumKitPiece::Tom1
            | DrumKitPiece::Tom2
            | DrumKitPiece::Tom3
            | DrumKitPiece::Tom4
            | DrumKitPiece::Tom5 => DrumKitPieceType::Tom,
            DrumKitPiece::Flexi1
            | DrumKitPiece::Flexi2
            | DrumKitPiece::Flexi3
            | DrumKitPiece::Flexi4 => DrumKitPieceType::Flexi,
        }
    }

    pub fn is_compatible(&self, other: &DrumVoice) -> bool {
        self.get_kit_piece_type() == other.get_kit_piece_type()
    }

    pub fn get_default_articulation(&self) -> DrumArticulation {
        Self::get_default_articulation_for_type(self.get_kit_piece_type())
    }

    pub fn get_default_articulation_for_type(kit_type: DrumKitPieceType) -> DrumArticulation {
        match kit_type {
            DrumKitPieceType::Kick | DrumKitPieceType::Tom | DrumKitPieceType::Snare => {
                DrumArticulation::DrumHead
            }
            DrumKitPieceType::HiHat => DrumArticulation::CymbalEdge,
            DrumKitPieceType::Crash => DrumArticulation::CymbalEdge,
            DrumKitPieceType::Ride => DrumArticulation::CymbalBow,
            DrumKitPieceType::Flexi => DrumArticulation::FlexiA,
            DrumKitPieceType::None => DrumArticulation::None,
        }
    }

    pub fn get_default_dimension_value(&self) -> f32 {
        if self.get_kit_piece_type() == DrumKitPieceType::HiHat {
            1.0
        } else {
            0.0
        }
    }
}

/// Represents a drum hit event
#[derive(Debug, Clone, Copy)]
pub struct DrumHit {
    pub voice: DrumVoice,
    pub velocity: f32,
    pub is_live: bool,
    pub dimension_value: f32,
}

impl fmt::Display for DrumHit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Voice: {} Velocity {}", self.voice, self.velocity)
    }
}

/// MIDI handler trait for processing note events
pub trait MidiHandler {
    fn handle_note_on(&mut self, channel: i32, note_number: i32, velocity: f32, sample_offset: i32);
    fn handle_poly_pressure(
        &mut self,
        channel: i32,
        note_number: i32,
        pressure: f32,
        sample_offset: i32,
    );
}

/// Drum MIDI device configuration
#[derive(Debug, Clone)]
pub struct DrumMidiDeviceConfiguration {
    pub name: String,
    pub hi_hat_pedal_channel: i32,
    pub hi_hat_pedal_closed: f32,
    pub hi_hat_pedal_semi_open: f32,
    pub hi_hat_pedal_open: f32,
    pub current_pedal_value: f32,
    pub snare_position_channel: i32,
    pub snare_position_center: f32,
    pub snare_position_edge: f32,
    pub snare_hot_spot_compensation: f32,

    midi_map: HashMap<i32, DrumVoice>,
}

impl DrumMidiDeviceConfiguration {
    pub fn generic() -> Self {
        let mut config = Self::new();
        config.name = "Generic".to_string();

        config.midi_map.insert(
            35,
            DrumVoice::new(DrumKitPiece::Kick, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            36,
            DrumVoice::new(DrumKitPiece::Kick, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            38,
            DrumVoice::new(DrumKitPiece::Snare, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            37,
            DrumVoice::new(DrumKitPiece::Snare, DrumArticulation::SideStick),
        );
        config.midi_map.insert(
            40,
            DrumVoice::new(DrumKitPiece::Snare, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            48,
            DrumVoice::new(DrumKitPiece::Tom1, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            45,
            DrumVoice::new(DrumKitPiece::Tom2, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            43,
            DrumVoice::new(DrumKitPiece::Tom3, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            47,
            DrumVoice::new(DrumKitPiece::Tom4, DrumArticulation::DrumHead),
        );
        config.midi_map.insert(
            46,
            DrumVoice::new(DrumKitPiece::HiHat, DrumArticulation::HiHatOpen),
        );
        config.midi_map.insert(
            42,
            DrumVoice::new(DrumKitPiece::HiHat, DrumArticulation::HiHatClosed),
        );
        config.midi_map.insert(
            44,
            DrumVoice::new(DrumKitPiece::HiHat, DrumArticulation::HiHatChick),
        );
        config.midi_map.insert(
            51,
            DrumVoice::new(DrumKitPiece::Ride, DrumArticulation::CymbalBow),
        );
        config.midi_map.insert(
            53,
            DrumVoice::new(DrumKitPiece::Ride, DrumArticulation::CymbalBell),
        );
        config.midi_map.insert(
            59,
            DrumVoice::new(DrumKitPiece::Ride, DrumArticulation::CymbalEdge),
        );
        config.midi_map.insert(
            49,
            DrumVoice::new(DrumKitPiece::Crash, DrumArticulation::CymbalEdge),
        );
        config.midi_map.insert(
            57,
            DrumVoice::new(DrumKitPiece::Crash2, DrumArticulation::CymbalEdge),
        );
        config.midi_map.insert(
            55,
            DrumVoice::new(DrumKitPiece::Crash3, DrumArticulation::CymbalEdge),
        );

        config
    }

    pub fn new() -> Self {
        Self {
            name: String::new(),
            hi_hat_pedal_channel: 4,
            hi_hat_pedal_closed: 1.0,
            hi_hat_pedal_semi_open: 0.5,
            hi_hat_pedal_open: 0.0,
            current_pedal_value: 0.0,
            snare_position_channel: 16,
            snare_position_center: 0.6,
            snare_position_edge: 0.8,
            snare_hot_spot_compensation: 0.0,
            midi_map: HashMap::new(),
        }
    }

    pub fn get_voice_from_midi_note(&self, midi_note: i32) -> DrumVoice {
        self.midi_map
            .get(&midi_note)
            .copied()
            .unwrap_or(DrumVoice::new(DrumKitPiece::None, DrumArticulation::None))
    }

    pub fn set_voice(&mut self, midi_note: i32, voice: DrumVoice) {
        self.midi_map.insert(midi_note, voice);
    }

    pub fn set_hi_hat_pedal_value(&mut self, pedal_value: f32) {
        self.current_pedal_value = pedal_value;
    }

    pub fn handle_note_on(
        &self,
        _channel: i32,
        note_number: i32,
        velocity: f32,
        _sample_offset: i32,
        is_live: bool,
    ) -> Option<DrumHit> {
        let voice = self.get_voice_from_midi_note(note_number);

        if voice.kit_piece != DrumKitPiece::None {
            let mut hit = DrumHit {
                voice,
                velocity,
                is_live,
                dimension_value: 0.0,
            };

            if is_live {
                if hit.voice.kit_piece == DrumKitPiece::HiHat {
                    hit.dimension_value = self.hi_hat_pedal_open
                        + (self.current_pedal_value
                            * (self.hi_hat_pedal_closed - self.hi_hat_pedal_open));
                } else if hit.voice.kit_piece == DrumKitPiece::Snare {
                    hit.dimension_value = self.snare_position_center;
                }
            } else {
                if hit.voice.kit_piece == DrumKitPiece::HiHat {
                    hit.dimension_value = if hit.voice.articulation == DrumArticulation::HiHatOpen {
                        0.0
                    } else {
                        1.0
                    };
                }
            }

            Some(hit)
        } else {
            None
        }
    }

    pub fn handle_poly_pressure(
        &self,
        _channel: i32,
        note_number: i32,
        pressure: f32,
        _sample_offset: i32,
        is_live: bool,
    ) -> Option<DrumHit> {
        let voice = self.get_voice_from_midi_note(note_number);

        if voice.kit_piece != DrumKitPiece::None {
            let ride_type = voice.kit_piece == DrumKitPiece::Ride
                || voice.kit_piece == DrumKitPiece::Crash
                || voice.kit_piece == DrumKitPiece::Crash2
                || voice.kit_piece == DrumKitPiece::Crash3;

            if ride_type {
                let hit = DrumHit {
                    voice: DrumVoice::new(voice.kit_piece, DrumArticulation::CymbalChoke),
                    velocity: pressure,
                    is_live,
                    dimension_value: 0.0,
                };
                return Some(hit);
            }
        }

        None
    }
}

impl Default for DrumMidiDeviceConfiguration {
    fn default() -> Self {
        Self::generic()
    }
}

/// Current map for MIDI device configuration
static CURRENT_MAP: OnceLock<RwLock<DrumMidiDeviceConfiguration>> = OnceLock::new();

fn current_map_lock() -> &'static RwLock<DrumMidiDeviceConfiguration> {
    CURRENT_MAP.get_or_init(|| RwLock::new(DrumMidiDeviceConfiguration::generic()))
}

/// Get the current MIDI map
pub fn get_current_map() -> DrumMidiDeviceConfiguration {
    current_map_lock()
        .read()
        .map(|map| map.clone())
        .unwrap_or_else(|_| DrumMidiDeviceConfiguration::generic())
}

/// Set the current MIDI map
pub fn set_current_map(map: DrumMidiDeviceConfiguration) {
    if let Ok(mut current_map) = current_map_lock().write() {
        *current_map = map;
    }
}

/// Note detector for pitch detection
#[derive(Debug, Clone)]
pub struct NoteDetector {
    pub max_frequency: f64,
    pub current_pitch: f32,

    valid_pitch_ratio: f32,
}

impl NoteDetector {
    pub fn new(_sample_rate: i32) -> Self {
        Self {
            max_frequency: 2637.0,
            current_pitch: 0.0,
            valid_pitch_ratio: (2.0_f32).powf(0.5 / 12.0),
        }
    }

    pub fn detect_note(&mut self, frequency: f64) -> bool {
        if frequency == 0.0 {
            return false;
        }

        let min_freq = frequency / self.valid_pitch_ratio as f64;
        let max_freq = frequency * self.valid_pitch_ratio as f64;

        self.current_pitch as f64 >= min_freq && self.current_pitch as f64 <= max_freq
    }

    pub fn stop(&mut self) {
        // Cleanup
    }
}

impl Default for NoteDetector {
    fn default() -> Self {
        Self::new(48000)
    }
}
