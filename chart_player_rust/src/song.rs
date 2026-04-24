//! Song data structures module
//!
//! Provides data structures for songs, notes, and song indexing.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Song index entry representing a single song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongIndexEntry {
    pub song_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub folder_path: String,
    pub arrangements: String,
    pub length_seconds: f32,
    pub lead_guitar_tuning: Option<String>,
    pub rhythm_guitar_tuning: Option<String>,
    pub bass_guitar_tuning: Option<String>,
    pub song_difficulty: Vec<f32>,
    #[serde(skip)]
    pub stats: Vec<Option<SongStatsEntry>>,
}

impl SongIndexEntry {
    pub fn new() -> Self {
        Self {
            song_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            folder_path: String::new(),
            arrangements: String::new(),
            length_seconds: 0.0,
            lead_guitar_tuning: None,
            rhythm_guitar_tuning: None,
            bass_guitar_tuning: None,
            song_difficulty: Vec::new(),
            stats: Vec::new(),
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for stat in &self.stats {
            if let Some(s) = stat {
                if let Some(ref tags) = s.tags {
                    if tags.contains(&tag.to_string()) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Default for SongIndexEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Song index for managing multiple songs
#[derive(Debug, Clone)]
pub struct SongIndex {
    pub songs: Vec<SongIndexEntry>,
    pub base_path: Option<String>,
    stats: Vec<SongStats>,
}

impl SongIndex {
    pub fn new(base_path: Option<&str>) -> Self {
        let base = base_path.map(|s| s.to_string());

        Self {
            songs: Vec::new(),
            base_path: base.clone(),
            stats: Vec::new(),
        }
    }

    pub fn load(base_path: &str, force_rescan: bool) -> std::io::Result<Self> {
        let mut index = Self::new(Some(base_path));

        let index_file = Path::new(base_path).join("index.json");

        if index_file.exists() && !force_rescan {
            if let Ok(contents) = fs::read_to_string(&index_file) {
                if let Ok(songs) = serde_json::from_str::<Vec<SongIndexEntry>>(&contents) {
                    index.songs = songs;
                }
            }
        } else if let Ok(_) = fs::create_dir_all(base_path) {
            index.scan_folder(base_path);
            index.save_index()?;
        }

        Ok(index)
    }

    fn scan_folder(&mut self, folder_path: &str) {
        let path = Path::new(folder_path);

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let song_file = path.join("song.json");
                    if song_file.exists() {
                        if let Ok(contents) = fs::read_to_string(&song_file) {
                            if let Ok(song_data) = serde_json::from_str::<SongData>(&contents) {
                                let entry = SongIndexEntry {
                                    song_name: song_data.song_name,
                                    artist_name: song_data.artist_name,
                                    album_name: song_data.album_name,
                                    folder_path: path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    arrangements: String::new(),
                                    length_seconds: song_data.song_length_seconds,
                                    lead_guitar_tuning: None,
                                    rhythm_guitar_tuning: None,
                                    bass_guitar_tuning: None,
                                    song_difficulty: vec![0.0; 7],
                                    stats: vec![None; 7],
                                };
                                self.songs.push(entry);
                            }
                        }
                    }
                    self.scan_folder(path.to_str().unwrap_or(""));
                }
            }
        }
    }

    fn save_index(&self) -> std::io::Result<()> {
        if let Some(ref base) = self.base_path {
            let index_file = Path::new(base).join("index.json");
            let contents = serde_json::to_string_pretty(&self.songs)?;
            fs::write(index_file, contents)?;
        }
        Ok(())
    }

    pub fn get_song_path(&self, entry: &SongIndexEntry) -> String {
        if let Some(ref base) = self.base_path {
            Path::new(base)
                .join(&entry.folder_path)
                .to_string_lossy()
                .to_string()
        } else {
            entry.folder_path.clone()
        }
    }
}

/// Song statistics entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongStatsEntry {
    pub song: String,
    pub last_played: Option<i64>,
    pub num_plays: i32,
    pub tags: Option<Vec<String>>,
}

impl SongStatsEntry {
    pub fn new(song: &str) -> Self {
        Self {
            song: song.to_string(),
            last_played: None,
            num_plays: 0,
            tags: None,
        }
    }

    pub fn add_tag(&mut self, tag: &str) {
        if self.tags.is_none() {
            self.tags = Some(Vec::new());
        }
        if let Some(ref mut tags) = self.tags {
            if !tags.contains(&tag.to_string()) {
                tags.push(tag.to_string());
            }
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(ref mut tags) = self.tags {
            tags.retain(|t| t != tag);
            if tags.is_empty() {
                self.tags = None;
            }
        }
    }
}

/// Song statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongStats {
    pub songs: Vec<SongStatsEntry>,
}

impl SongStats {
    pub fn new() -> Self {
        Self { songs: Vec::new() }
    }
}

/// Song data containing all information about a song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongData {
    pub song_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub song_length_seconds: f32,
    pub a440_cents_offset: f32,
    pub instrument_parts: Vec<SongInstrumentPart>,
}

impl SongData {
    pub fn new() -> Self {
        Self {
            song_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            song_length_seconds: 0.0,
            a440_cents_offset: 0.0,
            instrument_parts: Vec::new(),
        }
    }
}

/// Song instrument part (guitar, drums, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongInstrumentPart {
    pub instrument_type: SongInstrumentType,
    pub instrument_name: String,
    pub arrangement_name: Option<String>,
    pub song_audio: Option<String>,
    pub song_stem: Option<String>,
    pub tuning: Option<SongTuning>,
    pub capo_fret: i32,
    pub song_difficulty: f32,
}

/// Simplified chord/fingering description used by the fret scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongChord {
    pub name: Option<String>,
    pub fingers: Vec<i32>,
    pub frets: Vec<i32>,
}

impl SongChord {
    pub fn new(name: Option<&str>, fingers: Vec<i32>, frets: Vec<i32>) -> Self {
        Self {
            name: name.map(|value| value.to_string()),
            fingers,
            frets,
        }
    }
}

impl SongInstrumentPart {
    pub fn new(instrument_type: SongInstrumentType, name: &str) -> Self {
        Self {
            instrument_type,
            instrument_name: name.to_string(),
            arrangement_name: None,
            song_audio: None,
            song_stem: None,
            tuning: None,
            capo_fret: 0,
            song_difficulty: 0.0,
        }
    }
}

/// Song instrument type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SongInstrumentType {
    LeadGuitar = 0,
    RhythmGuitar = 1,
    BassGuitar = 2,
    Drums = 3,
    Keys = 4,
    Vocals = 5,
}

/// Song tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongTuning {
    pub string_semitone_offsets: Vec<i32>,
}

impl SongTuning {
    pub fn new() -> Self {
        Self {
            string_semitone_offsets: Vec::new(),
        }
    }

    pub fn is_offset_from_standard(&self) -> bool {
        self.string_semitone_offsets
            .get(1)
            .map(|&v| v != 0)
            .unwrap_or(false)
    }
}

/// Song note structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongNote {
    pub chord_id: i32,
    pub time_offset: f32,
    pub time_length: f32,
    pub end_time: f32,
    pub fret: i32,
    pub string: i32,
    pub techniques: i32,
    pub hand_fret: i32,
    pub finger_id: i32,
    pub slide_fret: i32,
    pub cents_offsets: Option<Vec<CentsOffset>>,
}

impl SongNote {
    pub fn new() -> Self {
        Self {
            chord_id: -1,
            time_offset: 0.0,
            time_length: 0.0,
            end_time: 0.0,
            fret: 0,
            string: 0,
            techniques: 0,
            hand_fret: 0,
            finger_id: -1,
            slide_fret: -1,
            cents_offsets: None,
        }
    }

    pub fn has_technique(&self, technique: i32) -> bool {
        self.techniques & technique != 0
    }
}

pub struct SongNoteTechnique;

impl SongNoteTechnique {
    pub const CHORD: i32 = 1 << 0;
    pub const CHORD_NOTE: i32 = 1 << 1;
    pub const CONTINUED: i32 = 1 << 2;
    pub const ACCENT: i32 = 1 << 3;
    pub const HAMMER_ON: i32 = 1 << 4;
    pub const PULL_OFF: i32 = 1 << 5;
    pub const FRET_HAND_MUTE: i32 = 1 << 6;
    pub const PALM_MUTE: i32 = 1 << 7;
    pub const HARMONIC: i32 = 1 << 8;
    pub const PINCH_HARMONIC: i32 = 1 << 9;
    pub const SLIDE: i32 = 1 << 10;
    pub const VIBRATO: i32 = 1 << 11;
    pub const BEND: i32 = 1 << 12;
}

/// Drum note structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongDrumNote {
    pub time_offset: f32,
    pub time_length: f32,
    pub end_time: f32,
    pub kit_piece: DrumKitPieceSimple,
    pub articulation: DrumArticulationSimple,
}

impl SongDrumNote {
    pub fn new() -> Self {
        Self {
            time_offset: 0.0,
            time_length: 0.0,
            end_time: 0.0,
            kit_piece: DrumKitPieceSimple::Kick,
            articulation: DrumArticulationSimple::DrumHead,
        }
    }
}

/// Simplified drum kit piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrumKitPieceSimple {
    Kick,
    Snare,
    HiHat,
    Crash,
    Ride,
    Tom1,
    Tom2,
    Tom3,
}

impl DrumKitPieceSimple {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => DrumKitPieceSimple::Kick,
            1 => DrumKitPieceSimple::Snare,
            2 => DrumKitPieceSimple::HiHat,
            3 => DrumKitPieceSimple::Crash,
            4 => DrumKitPieceSimple::Ride,
            5 => DrumKitPieceSimple::Tom1,
            6 => DrumKitPieceSimple::Tom2,
            7 => DrumKitPieceSimple::Tom3,
            _ => DrumKitPieceSimple::Kick,
        }
    }
}

/// Simplified drum articulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrumArticulationSimple {
    DrumHead,
    Rim,
    HiHatClosed,
    HiHatOpen,
    CymbalEdge,
    CymbalBow,
    CymbalBell,
}

impl DrumArticulationSimple {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => DrumArticulationSimple::DrumHead,
            1 => DrumArticulationSimple::Rim,
            2 => DrumArticulationSimple::HiHatClosed,
            3 => DrumArticulationSimple::HiHatOpen,
            4 => DrumArticulationSimple::CymbalEdge,
            5 => DrumArticulationSimple::CymbalBow,
            6 => DrumArticulationSimple::CymbalBell,
            _ => DrumArticulationSimple::DrumHead,
        }
    }
}

/// Keyboard note structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongKeyboardNote {
    pub time_offset: f32,
    pub time_length: f32,
    pub end_time: f32,
    pub note: i32,
}

impl SongKeyboardNote {
    pub fn new() -> Self {
        Self {
            time_offset: 0.0,
            time_length: 0.0,
            end_time: 0.0,
            note: 60,
        }
    }
}

/// Song structure (beats, sections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongStructure {
    pub beats: Vec<SongBeat>,
    pub sections: Vec<SongSection>,
}

impl SongStructure {
    pub fn new() -> Self {
        Self {
            beats: Vec::new(),
            sections: Vec::new(),
        }
    }
}

/// Song beat marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongBeat {
    pub time_offset: f32,
    pub is_measure: bool,
}

impl SongBeat {
    pub fn new(time: f32, measure: bool) -> Self {
        Self {
            time_offset: time,
            is_measure: measure,
        }
    }
}

/// Song section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSection {
    pub name: Option<String>,
    pub start_time: f32,
    pub end_time: f32,
}

impl SongSection {
    pub fn new(name: Option<&str>, start: f32, end: f32) -> Self {
        Self {
            name: name.map(|s| s.to_string()),
            start_time: start,
            end_time: end,
        }
    }
}

/// Cents offset for bend effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentsOffset {
    pub time_offset: f32,
    pub cents: i32,
}

impl CentsOffset {
    pub fn new(time: f32, cents: i32) -> Self {
        Self {
            time_offset: time,
            cents,
        }
    }
}

/// Song vocal lyric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongVocal {
    pub time_offset: f32,
    pub vocal: String,
}

impl SongVocal {
    pub fn new() -> Self {
        Self {
            time_offset: 0.0,
            vocal: String::new(),
        }
    }
}

/// Song tuning mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SongTuningMode {
    None = 0,
    A440 = 1,
    EStandard = 2,
    EbStandard = 3,
    DStandard = 4,
    CSharpStandard = 5,
    CStandard = 6,
    BStandard = 7,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: std::path::PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("chart_player_song_tests_{unique}"));
            fs::create_dir_all(&path).expect("failed to create test directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_song_json(path: &Path, song_name: &str, artist_name: &str) {
        let song = SongData {
            song_name: song_name.to_string(),
            artist_name: artist_name.to_string(),
            album_name: "Test Album".to_string(),
            song_length_seconds: 123.5,
            a440_cents_offset: 0.0,
            instrument_parts: vec![SongInstrumentPart {
                instrument_type: SongInstrumentType::LeadGuitar,
                instrument_name: "lead".to_string(),
                arrangement_name: Some("Lead".to_string()),
                song_audio: Some("song.ogg".to_string()),
                song_stem: Some("lead.ogg".to_string()),
                tuning: Some(SongTuning {
                    string_semitone_offsets: vec![0, 0, 0, 0, 0, 0],
                }),
                capo_fret: 0,
                song_difficulty: 2.5,
            }],
        };
        let contents = serde_json::to_string(&song).expect("failed to serialize song data");
        fs::write(path.join("song.json"), contents).expect("failed to write song.json");
    }

    #[test]
    fn load_scans_song_folder_and_writes_index_json() {
        let temp = TestDir::new();
        let song_folder = temp.path().join("My Song");
        fs::create_dir_all(&song_folder).expect("failed to create song folder");
        write_song_json(&song_folder, "My Song", "Test Artist");

        let index = SongIndex::load(temp.path().to_str().expect("invalid temp path"), true)
            .expect("failed to load song index");

        assert_eq!(index.songs.len(), 1);
        assert_eq!(index.songs[0].song_name, "My Song");
        assert_eq!(index.songs[0].artist_name, "Test Artist");
        assert_eq!(index.songs[0].folder_path, "My Song");
        assert!(temp.path().join("index.json").exists());
    }

    #[test]
    fn load_uses_existing_index_file_without_rescan() {
        let temp = TestDir::new();
        let entry = SongIndexEntry {
            song_name: "Indexed Song".to_string(),
            artist_name: "Indexed Artist".to_string(),
            album_name: "Indexed Album".to_string(),
            folder_path: "indexed/song".to_string(),
            arrangements: "L".to_string(),
            length_seconds: 150.0,
            lead_guitar_tuning: Some("E Standard".to_string()),
            rhythm_guitar_tuning: None,
            bass_guitar_tuning: None,
            song_difficulty: vec![1.0, 0.0, 0.0],
            stats: Vec::new(),
        };
        let contents = serde_json::to_string(&vec![entry]).expect("failed to serialize index");
        fs::write(temp.path().join("index.json"), contents).expect("failed to write index.json");

        let index = SongIndex::load(temp.path().to_str().expect("invalid temp path"), false)
            .expect("failed to load existing index");

        assert_eq!(index.songs.len(), 1);
        assert_eq!(index.songs[0].song_name, "Indexed Song");
        assert_eq!(index.songs[0].folder_path, "indexed/song");
    }

    #[test]
    fn get_song_path_joins_base_and_relative_folder() {
        let index = SongIndex::new(Some("/tmp/chartplayer"));
        let entry = SongIndexEntry {
            song_name: "Song".to_string(),
            artist_name: "Artist".to_string(),
            album_name: "Album".to_string(),
            folder_path: "set/song".to_string(),
            arrangements: String::new(),
            length_seconds: 60.0,
            lead_guitar_tuning: None,
            rhythm_guitar_tuning: None,
            bass_guitar_tuning: None,
            song_difficulty: Vec::new(),
            stats: Vec::new(),
        };

        let song_path = index.get_song_path(&entry);

        assert!(song_path.ends_with("/tmp/chartplayer/set/song") || song_path.ends_with(r"\tmp\chartplayer\set\song"));
    }
}
