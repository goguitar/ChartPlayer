//! Note utility functions for pitch calculations

use std::f64::consts::PI;

/// Note name enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteName {
    C,
    CsDf,
    D,
    DsEf,
    E,
    F,
    FsGf,
    G,
    GsAf,
    A,
    AsBf,
    B,
}

impl NoteName {
    pub fn from_midi(midi: i32) -> Self {
        match midi % 12 {
            0 => NoteName::C,
            1 => NoteName::CsDf,
            2 => NoteName::D,
            3 => NoteName::DsEf,
            4 => NoteName::E,
            5 => NoteName::F,
            6 => NoteName::FsGf,
            7 => NoteName::G,
            8 => NoteName::GsAf,
            9 => NoteName::A,
            10 => NoteName::AsBf,
            11 => NoteName::B,
            _ => NoteName::C,
        }
    }
    
    pub fn to_string(&self) -> &'static str {
        match self {
            NoteName::C => "C",
            NoteName::CsDf => "Cs/Df",
            NoteName::D => "D",
            NoteName::DsEf => "Ds/Ef",
            NoteName::E => "E",
            NoteName::F => "F",
            NoteName::FsGf => "Fs/Gf",
            NoteName::G => "G",
            NoteName::GsAf => "Gs/Af",
            NoteName::A => "A",
            NoteName::AsBf => "As/Bf",
            NoteName::B => "B",
        }
    }
}

/// Chord type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordType {
    Maj,
    Min,
}

/// Note utility functions
pub struct NoteUtil;

impl NoteUtil {
    /// Scale intervals for different chord types
    pub fn scales(chord_type: ChordType) -> &'static [i32] {
        match chord_type {
            ChordType::Maj => &[0, 2, 4, 5, 7, 9, 11],
            ChordType::Min => &[0, 2, 3, 5, 7, 8, 11],
        }
    }
    
    /// Calculate the frequency of a MIDI note number
    pub fn midi_note_frequency(midi_note: f64) -> f64 {
        let a4_midi = 57.0;
        let a4_freq = 440.0;
        let half_step_ratio = 2.0_f64.powf(1.0 / 12.0);
        a4_freq / half_step_ratio.powf(a4_midi - midi_note)
    }
    
    /// Get the note name from a MIDI note number
    pub fn note_name(midi_note: i32) -> NoteName {
        NoteName::from_midi(midi_note)
    }
    
    /// Get the octave from a MIDI note number
    pub fn note_octave(midi_note: i32) -> i32 {
        (midi_note / 12) - 1
    }
    
    /// Get the MIDI note number from note name and octave
    pub fn midi_note_number(note: NoteName, octave: i32) -> i32 {
        ((octave + 1) * 12) + note as i32
    }
    
    /// Calculate the semitone difference between two frequencies
    pub fn semitone_difference(freq1: f64, freq2: f64) -> f64 {
        12.0 * (freq1 / freq2).log2()
    }
    
    /// Parse a note string (e.g., "A4", "C#3") into frequency
    pub fn parse_note_frequency(note_str: &str) -> Option<f64> {
        let note_str = note_str.trim().to_lowercase();
        
        if note_str.is_empty() {
            return None;
        }
        
        let (note_part, octave_part) = if note_str.len() >= 2 {
            (&note_str[..note_str.len() - 1], &note_str[note_str.len() - 1..])
        } else {
            return None;
        };
        
        let octave: i32 = octave_part.parse().ok()?;
        
        let note = match note_part {
            "c" => NoteName::C,
            "cs" | "df" => NoteName::CsDf,
            "d" => NoteName::D,
            "ds" | "ef" => NoteName::DsEf,
            "e" => NoteName::E,
            "f" => NoteName::F,
            "fs" | "gf" => NoteName::FsGf,
            "g" => NoteName::G,
            "gs" | "af" => NoteName::GsAf,
            "a" => NoteName::A,
            "as" | "bf" => NoteName::AsBf,
            "b" => NoteName::B,
            _ => return None,
        };
        
        Some(Self::midi_note_frequency(Self::midi_note_number(note, octave) as f64))
    }
}