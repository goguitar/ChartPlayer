//! Audio module for song playback
//! 
//! Provides audio playback, resampling, and mixing functionality.

use anyhow::{Context, Result};
use crate::song::{
    DrumArticulationSimple, DrumKitPieceSimple, SongBeat, SongChord, SongDrumNote,
    SongInstrumentType, SongKeyboardNote, SongNote, SongNoteTechnique, SongStructure,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fmt::Debug;
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Sample history buffer for FFT processing
pub struct SampleHistory<T> {
    data: Vec<T>,
    current_offset: usize,
}

impl<T: Copy + Default> SampleHistory<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_offset: 0,
        }
    }
    
    pub fn set_size(&mut self, size: usize) {
        if self.data.len() != size {
            self.data.resize(size, T::default());
        }
    }
    
    pub fn copy_from(&mut self, source: &[T]) {
        let mut left = source.len();
        let mut offset = 0;
        
        while left > 0 {
            let to_copy = left.min(self.data.len() - self.current_offset);
            self.data[self.current_offset..self.current_offset + to_copy].copy_from_slice(&source[offset..offset + to_copy]);
            self.current_offset = (self.current_offset + to_copy) % self.data.len();
            left -= to_copy;
            offset += to_copy;
        }
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    pub fn current_offset(&self) -> usize {
        self.current_offset
    }
}

impl Default for SampleHistory<f32> {
    fn default() -> Self {
        Self::new()
    }
}

/// Vorbis mixer for reading OGG audio files
pub struct VorbisMixer {
    sample_rate: i32,
    channels: i32,
    total_samples: u64,
    total_time: Duration,
}

impl Clone for VorbisMixer {
    fn clone(&self) -> Self {
        Self {
            sample_rate: self.sample_rate,
            channels: self.channels,
            total_samples: self.total_samples,
            total_time: self.total_time,
        }
    }
}

impl Debug for VorbisMixer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VorbisMixer")
            .field("sample_rate", &self.sample_rate)
            .field("channels", &self.channels)
            .field("total_samples", &self.total_samples)
            .finish()
    }
}

impl VorbisMixer {
    pub fn new(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            sample_rate: 48000,
            channels: 2,
            total_samples: 0,
            total_time: Duration::from_secs(0),
        })
    }
    
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }
    
    pub fn channels(&self) -> i32 {
        self.channels
    }
    
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
    
    pub fn total_time(&self) -> Duration {
        self.total_time
    }
    
    pub fn read_samples(&mut self, buffer: &mut [f32], offset: usize, count: usize) -> usize {
        let available = count.min(buffer.len() - offset);
        for i in 0..available {
            buffer[offset + i] = 0.0;
        }
        available
    }
}

/// WDL-style resampler for audio rate conversion
pub struct WdlResampler {
    srate_in: f64,
    srate_out: f64,
    ratio: f64,
    frac_pos: f64,
    filter_pos: f32,
    filter_q: f32,
    lp_over_size: i32,
    sinc_size: i32,
    sinc_over_size: i32,
    filter_cnt: i32,
    interp: bool,
    feed_mode: bool,
    filter_coeffs: Vec<f32>,
    rs_in_buf: Vec<f32>,
    samples_in_rs_in_buf: usize,
    last_requested: usize,
    filter_latency: usize,
    iir_filter: Option<WdlResamplerIirFilter>,
}

impl Clone for WdlResampler {
    fn clone(&self) -> Self {
        Self {
            srate_in: self.srate_in,
            srate_out: self.srate_out,
            ratio: self.ratio,
            frac_pos: self.frac_pos,
            filter_pos: self.filter_pos,
            filter_q: self.filter_q,
            lp_over_size: self.lp_over_size,
            sinc_size: self.sinc_size,
            sinc_over_size: self.sinc_over_size,
            filter_cnt: self.filter_cnt,
            interp: self.interp,
            feed_mode: self.feed_mode,
            filter_coeffs: self.filter_coeffs.clone(),
            rs_in_buf: self.rs_in_buf.clone(),
            samples_in_rs_in_buf: self.samples_in_rs_in_buf,
            last_requested: self.last_requested,
            filter_latency: self.filter_latency,
            iir_filter: self.iir_filter.clone(),
        }
    }
}

impl Debug for WdlResampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WdlResampler")
            .field("srate_in", &self.srate_in)
            .field("srate_out", &self.srate_out)
            .field("ratio", &self.ratio)
            .finish()
    }
}

impl WdlResampler {
    pub fn new() -> Self {
        Self {
            srate_in: 44100.0,
            srate_out: 44100.0,
            ratio: 1.0,
            frac_pos: 0.0,
            filter_pos: 0.693,
            filter_q: 0.707,
            lp_over_size: 1,
            sinc_size: 0,
            sinc_over_size: 0,
            filter_cnt: 1,
            interp: true,
            feed_mode: false,
            filter_coeffs: Vec::new(),
            rs_in_buf: Vec::new(),
            samples_in_rs_in_buf: 0,
            last_requested: 0,
            filter_latency: 0,
            iir_filter: None,
        }
    }
    
    pub fn set_mode(&mut self, interp: bool, filter_cnt: i32, sinc: bool, sinc_size: i32, sinc_interp_size: i32) {
        self.sinc_size = if sinc && sinc_size >= 4 { sinc_size.min(8192) } else { 0 };
        self.sinc_over_size = if self.sinc_size != 0 { sinc_interp_size.clamp(1, 4096) } else { 1 };
        self.filter_cnt = if self.sinc_size != 0 { 0 } else { filter_cnt.clamp(0, 4) };
        self.interp = interp && (self.sinc_size == 0);
        
        if self.sinc_size == 0 {
            self.filter_coeffs.clear();
        }
        if self.filter_cnt == 0 {
            self.iir_filter = None;
        }
    }
    
    pub fn set_filter_parms(&mut self, filter_pos: f32, filter_q: f32) {
        self.filter_pos = filter_pos;
        self.filter_q = filter_q;
    }
    
    pub fn set_feed_mode(&mut self, want_input_driven: bool) {
        self.feed_mode = want_input_driven;
    }
    
    pub fn reset(&mut self, frac_pos: f64) {
        self.last_requested = 0;
        self.filter_latency = 0;
        self.frac_pos = frac_pos;
        self.samples_in_rs_in_buf = 0;
        if let Some(ref mut filter) = self.iir_filter {
            filter.reset();
        }
    }
    
    pub fn set_rates(&mut self, rate_in: f64, rate_out: f64) {
        if rate_in < 1.0 || rate_out < 1.0 {
            return;
        }
        if self.srate_in != rate_in || self.srate_out != rate_out {
            self.srate_in = rate_in;
            self.srate_out = rate_out;
            self.ratio = self.srate_in / self.srate_out;
        }
    }
    
    pub fn get_current_latency(&self) -> f64 {
        let latency = (self.samples_in_rs_in_buf as f64 - self.filter_latency as f64) / self.srate_in;
        latency.max(0.0)
    }
    
    pub fn resample_prepare(&mut self, out_samples: i32, nch: i32) -> Option<(&mut [f32], usize)> {
        if nch > 64 || nch < 1 {
            return None;
        }
        
        let fsize = if self.sinc_size > 1 { self.sinc_size } else { 0 };
        let hfs = fsize / 2;
        
        if hfs > 1 && self.samples_in_rs_in_buf < hfs as usize - 1 {
            self.filter_latency += hfs as usize - 1 - self.samples_in_rs_in_buf;
            self.samples_in_rs_in_buf = hfs as usize - 1;
        }
        
        let sreq = if !self.feed_mode {
            (self.ratio * out_samples as f64) as i32 + 4 + fsize - self.samples_in_rs_in_buf as i32
        } else {
            out_samples
        };
        let sreq = sreq.max(0) as usize;
        
        let needed = self.samples_in_rs_in_buf + sreq;
        self.rs_in_buf.resize(needed * nch as usize, 0.0);
        
        self.last_requested = sreq;
        
        Some((&mut self.rs_in_buf, self.samples_in_rs_in_buf * nch as usize))
    }
    
    pub fn resample_out(&mut self, out_buffer: &mut [f32], out_buffer_index: usize, 
                        nsamples_in: i32, nsamples_out: i32, nch: i32) -> i32 {
        if nch > 64 || nch < 1 {
            return 0;
        }
        
        self.samples_in_rs_in_buf += (nsamples_in as usize).min(self.last_requested);
        
        let mut ret = 0;
        let mut src_pos = self.frac_pos;
        let drs_pos = self.ratio;
        let local_in = 0;
        let mut out_ptr = out_buffer_index;
        let mut ns = nsamples_out;
        
        if self.interp {
            while ns > 0 {
                let ipos = src_pos as i32;
                if ipos >= self.samples_in_rs_in_buf as i32 - 1 {
                    break;
                }
                
                let frac = src_pos - ipos as f64;
                let in_ptr = (local_in + ipos as usize) * nch as usize;
                
                for c in 0..nch as usize {
                    let a = self.rs_in_buf[in_ptr + c];
                    let b = self.rs_in_buf[in_ptr + nch as usize + c];
                    out_buffer[out_ptr + c] = (a as f32 * (1.0 - frac) as f32 + b as f32 * frac as f32);
                }
                
                out_ptr += nch as usize;
                src_pos += drs_pos;
                ns -= 1;
                ret += 1;
            }
        }
        
        let isrc_pos = src_pos as i32;
        self.frac_pos = src_pos - isrc_pos as f64;
        self.samples_in_rs_in_buf = self.samples_in_rs_in_buf.saturating_sub(isrc_pos as usize);
        
        ret
    }
}

impl Default for WdlResampler {
    fn default() -> Self {
        Self::new()
    }
}

/// IIR filter for WDL resampler
#[derive(Debug, Clone)]
struct WdlResamplerIirFilter {
    fpos: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    hist: Vec<[f64; 4]>,
}

impl WdlResamplerIirFilter {
    fn new() -> Self {
        Self {
            fpos: -1.0,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            hist: vec![[0.0; 4]; 256],
        }
    }
    
    fn reset(&mut self) {
        for h in &mut self.hist {
            *h = [0.0; 4];
        }
    }
    
    fn set_parms(&mut self, fpos: f64, q: f64) {
        if (self.fpos - fpos).abs() < 0.000001 {
            return;
        }
        self.fpos = fpos;
        
        let pos = fpos * std::f64::consts::PI;
        let cpos = pos.cos();
        let spos = pos.sin();
        
        let alpha = spos / (2.0 * q);
        let sc = 1.0 / (1.0 + alpha);
        
        self.b1 = (1.0 - cpos) * sc;
        let half_b1 = self.b1 * 0.5;
        self.b0 = half_b1;
        self.b2 = half_b1;
        self.a1 = -2.0 * cpos * sc;
        self.a2 = (1.0 - alpha) * sc;
    }
}

pub struct AudioOutput {
    _stream: cpal::Stream,
    player: SharedSongPlayer,
}

pub type SharedSongPlayer = Arc<Mutex<SongPlayer>>;

impl AudioOutput {
    pub fn new(mut player: SongPlayer) -> Result<Self> {
        Self::from_shared(Arc::new(Mutex::new({
            player.paused = false;
            player
        })))
    }

    pub fn from_shared(player: SharedSongPlayer) -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device().context("failed to find default audio output device")?;
        let supported_config = device
            .default_output_config()
            .context("failed to query default audio output config")?;
        let stream_config: cpal::StreamConfig = supported_config.config();
        if let Ok(mut locked_player) = player.lock() {
            locked_player.paused = false;
            locked_player.finished_playing = false;
            locked_player.set_playback_sample_rate(stream_config.sample_rate.0 as f64);
        }

        let callback_player = Arc::clone(&player);
        let err_fn = |err| eprintln!("audio stream error: {err}");

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => build_output_stream::<f32>(&device, &stream_config, callback_player, err_fn)?,
            cpal::SampleFormat::I16 => build_output_stream::<i16>(&device, &stream_config, callback_player, err_fn)?,
            cpal::SampleFormat::U16 => build_output_stream::<u16>(&device, &stream_config, callback_player, err_fn)?,
            sample_format => anyhow::bail!("unsupported audio sample format: {sample_format:?}"),
        };

        stream.play().context("failed to start audio output stream")?;

        Ok(Self {
            _stream: stream,
            player,
        })
    }

    pub fn player(&self) -> SharedSongPlayer {
        Arc::clone(&self.player)
    }
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    player: SharedSongPlayer,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;

    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_output_data(data, channels, &player),
            err_fn,
            None,
        )
        .context("failed to build audio output stream")
}

fn write_output_data<T>(output: &mut [T], channels: usize, player: &SharedSongPlayer)
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    if channels == 0 {
        return;
    }

    let frames = output.len() / channels;
    let mut left = vec![0.0_f32; frames];
    let mut right = vec![0.0_f32; frames];

    if let Ok(mut player) = player.lock() {
        player.read_samples(&mut left, &mut right);
    }

    for (frame_index, frame) in output.chunks_mut(channels).enumerate() {
        let left_sample = left.get(frame_index).copied().unwrap_or(0.0);
        let right_sample = right.get(frame_index).copied().unwrap_or(left_sample);

        for (channel_index, sample) in frame.iter_mut().enumerate() {
            let value = match channel_index {
                0 => left_sample,
                1 => right_sample,
                _ => (left_sample + right_sample) * 0.5,
            };
            *sample = T::from_sample(value);
        }
    }
}

/// Song player for audio playback
#[derive(Debug, Clone)]
pub struct SongPlayer {
    pub base_path: Option<String>,
    pub playback_sample_rate: f64,
    pub song_sample_rate: f64,
    pub current_second: f32,
    pub playback_speed: f32,
    pub song_length_seconds: f32,
    pub paused: bool,
    pub finished_playing: bool,
    pub song_tuning_mode: i32,
    pub tuning_offset_semitones: f64,
    pub pitch_shift_semitones: f64,
    
    pub loudness: Vec<f32>,
    pub loaded_loudness: usize,
    pub song_rms: f32,
    pub mute_part_stems: bool,
    pub instrument_type: SongInstrumentType,
    pub song_structure: SongStructure,
    pub chords: Vec<SongChord>,
    pub instrument_notes: Vec<SongNote>,
    pub drum_notes: Vec<SongDrumNote>,
    pub keyboard_notes: Vec<SongKeyboardNote>,
    
    vorbis_mixer: Option<VorbisMixer>,
    resampler: WdlResampler,
    seek_time: Option<f32>,
    total_samples: i64,
    current_playback_sample: i64,
    sample_data: Vec<Vec<f32>>,
    pitch_shift: f64,
    linear_gain: f32,
}

impl SongPlayer {
    pub fn new() -> Self {
        Self::demo_for(SongInstrumentType::LeadGuitar)
    }

    pub fn shared_demo_for(instrument_type: SongInstrumentType) -> SharedSongPlayer {
        Arc::new(Mutex::new(Self::demo_for(instrument_type)))
    }

    pub fn demo_for(instrument_type: SongInstrumentType) -> Self {
        let mut player = Self::empty(instrument_type);
        player.populate_demo_content();
        player
    }

    fn empty(instrument_type: SongInstrumentType) -> Self {
        Self {
            base_path: None,
            playback_sample_rate: 48000.0,
            song_sample_rate: 0.0,
            current_second: 0.0,
            playback_speed: 1.0,
            song_length_seconds: 0.0,
            paused: false,
            finished_playing: false,
            song_tuning_mode: 1,
            tuning_offset_semitones: 0.0,
            pitch_shift_semitones: 0.0,
            loudness: vec![0.0; 512],
            loaded_loudness: 0,
            song_rms: 0.0,
            mute_part_stems: false,
            instrument_type,
            song_structure: SongStructure::new(),
            chords: Vec::new(),
            instrument_notes: Vec::new(),
            drum_notes: Vec::new(),
            keyboard_notes: Vec::new(),
            vorbis_mixer: None,
            resampler: WdlResampler::new(),
            seek_time: None,
            total_samples: 0,
            current_playback_sample: 0,
            sample_data: vec![Vec::new(), Vec::new()],
            pitch_shift: 1.0,
            linear_gain: 1.0,
        }
    }

    fn populate_demo_content(&mut self) {
        self.song_sample_rate = 48_000.0;
        self.playback_sample_rate = self.song_sample_rate;
        self.song_length_seconds = 16.0;
        self.song_structure.beats = (0..32)
            .map(|beat| SongBeat::new(beat as f32 * 0.5, beat % 4 == 0))
            .collect();

        self.chords = vec![
            SongChord::new(Some("G"), vec![2, 1, 0, 0, 0, 3], vec![3, 2, 0, 0, 0, 3]),
            SongChord::new(Some("D"), vec![-1, -1, 0, 2, 3, 2], vec![-1, -1, 0, 2, 3, 2]),
            SongChord::new(Some("Em"), vec![0, 2, 2, 0, 0, 0], vec![0, 2, 2, 0, 0, 0]),
        ];

        self.instrument_notes = vec![
            demo_note_with(0.5, 1.2, 3, 5, 4, SongNoteTechnique::ACCENT, -1, -1, -1, None),
            demo_note_with(1.2, 0.8, 5, 4, 4, SongNoteTechnique::HAMMER_ON, -1, -1, -1, None),
            demo_note_with(1.9, 1.2, 3, 5, 2, SongNoteTechnique::CHORD | SongNoteTechnique::ACCENT, 0, 0, -1, None),
            demo_note_with(3.5, 0.9, 7, 3, 5, SongNoteTechnique::SLIDE, -1, -1, 10, None),
            demo_note_with(4.6, 0.9, 10, 1, 7, SongNoteTechnique::VIBRATO, -1, -1, -1, None),
            demo_note_with(5.4, 1.2, 2, 3, 2, SongNoteTechnique::CHORD | SongNoteTechnique::PALM_MUTE, 1, 1, -1, None),
            demo_note_with(6.9, 0.9, 0, 2, 7, SongNoteTechnique::HARMONIC, -1, -1, -1, None),
            demo_note_with(7.8, 1.0, 7, 2, 6, SongNoteTechnique::BEND, -1, -1, -1, Some(vec![(7.8, 0), (8.2, 120), (8.8, 120)])),
            demo_note_with(9.2, 0.8, 9, 3, 7, SongNoteTechnique::PULL_OFF, -1, -1, -1, None),
            demo_note_with(10.0, 1.2, 0, 5, 0, SongNoteTechnique::CHORD | SongNoteTechnique::FRET_HAND_MUTE, 2, 2, -1, None),
            demo_note_with(11.5, 0.9, 12, 5, 10, SongNoteTechnique::PINCH_HARMONIC | SongNoteTechnique::ACCENT, -1, -1, -1, None),
            demo_note_with(12.6, 0.9, 14, 2, 11, SongNoteTechnique::SLIDE, -1, -1, 17, None),
            demo_note_with(13.4, 0.9, 15, 1, 12, SongNoteTechnique::PALM_MUTE, -1, -1, -1, None),
            demo_note_with(14.2, 1.2, 17, 0, 14, SongNoteTechnique::ACCENT, -1, -1, -1, None),
        ];

        self.drum_notes = vec![
            demo_drum(0.0, DrumKitPieceSimple::Kick),
            demo_drum(0.5, DrumKitPieceSimple::HiHat),
            demo_drum(1.0, DrumKitPieceSimple::Snare),
            demo_drum(1.5, DrumKitPieceSimple::HiHat),
            demo_drum(2.0, DrumKitPieceSimple::Kick),
            demo_drum(2.5, DrumKitPieceSimple::Crash),
            demo_drum(3.0, DrumKitPieceSimple::Snare),
            demo_drum(3.5, DrumKitPieceSimple::Ride),
            demo_drum(4.0, DrumKitPieceSimple::Kick),
            demo_drum(4.5, DrumKitPieceSimple::Tom1),
            demo_drum(5.0, DrumKitPieceSimple::Tom2),
            demo_drum(5.5, DrumKitPieceSimple::Tom3),
            demo_drum(6.0, DrumKitPieceSimple::Snare),
            demo_drum(6.5, DrumKitPieceSimple::Crash),
            demo_drum(7.0, DrumKitPieceSimple::Kick),
            demo_drum(7.5, DrumKitPieceSimple::Ride),
        ];

        self.keyboard_notes = vec![
            demo_key(0.3, 1.2, 52),
            demo_key(0.6, 1.1, 55),
            demo_key(0.9, 1.0, 59),
            demo_key(2.0, 0.9, 60),
            demo_key(2.3, 0.9, 64),
            demo_key(2.6, 0.9, 67),
            demo_key(4.0, 1.2, 57),
            demo_key(4.4, 1.0, 60),
            demo_key(4.8, 1.0, 64),
            demo_key(6.0, 1.1, 59),
            demo_key(6.4, 1.0, 62),
            demo_key(6.8, 1.0, 66),
            demo_key(8.2, 1.2, 55),
            demo_key(8.5, 1.0, 59),
            demo_key(8.8, 1.0, 62),
            demo_key(10.0, 1.1, 60),
            demo_key(10.3, 1.1, 64),
            demo_key(10.6, 1.1, 67),
        ];

        self.generate_demo_audio();
    }

    fn generate_demo_audio(&mut self) {
        let sample_rate = self.song_sample_rate.max(1.0) as usize;
        let total_samples = (self.song_length_seconds.max(0.0) * sample_rate as f32) as usize;
        self.sample_data = vec![vec![0.0; total_samples], vec![0.0; total_samples]];
        self.total_samples = total_samples as i64;
        self.current_playback_sample = 0;
        self.finished_playing = false;

        match self.instrument_type {
            SongInstrumentType::Drums => self.render_demo_drums(sample_rate),
            SongInstrumentType::Keys => self.render_demo_keys(sample_rate),
            _ => self.render_demo_strings(sample_rate),
        }
    }

    fn render_demo_strings(&mut self, sample_rate: usize) {
        let notes = self.instrument_notes.clone();
        for note in &notes {
            self.render_string_note(note, sample_rate);
        }
    }

    fn render_demo_keys(&mut self, sample_rate: usize) {
        let notes = self.keyboard_notes.clone();
        for note in notes {
            self.render_sine_note(
                midi_frequency(note.note),
                note.time_offset,
                note.time_length,
                0.08,
                0.2,
                sample_rate,
            );
        }
    }

    fn render_demo_drums(&mut self, sample_rate: usize) {
        let notes = self.drum_notes.clone();
        for note in notes {
            let (frequency, decay, gain) = match note.kit_piece {
                DrumKitPieceSimple::Kick => (60.0, 0.18, 0.22),
                DrumKitPieceSimple::Snare => (180.0, 0.10, 0.14),
                DrumKitPieceSimple::HiHat => (360.0, 0.05, 0.09),
                DrumKitPieceSimple::Crash => (280.0, 0.28, 0.10),
                DrumKitPieceSimple::Ride => (420.0, 0.22, 0.09),
                DrumKitPieceSimple::Tom1 => (140.0, 0.12, 0.12),
                DrumKitPieceSimple::Tom2 => (120.0, 0.14, 0.12),
                DrumKitPieceSimple::Tom3 => (100.0, 0.16, 0.12),
            };
            self.render_sine_note(frequency, note.time_offset, decay, gain, 0.0, sample_rate);
        }
    }

    fn render_string_note(&mut self, note: &SongNote, sample_rate: usize) {
        let start_sample = (note.time_offset.max(0.0) * sample_rate as f32) as usize;
        let sample_count = (note.time_length.max(0.0) * sample_rate as f32) as usize;
        if start_sample >= self.sample_data[0].len() || sample_count == 0 {
            return;
        }

        let end_sample = (start_sample + sample_count).min(self.sample_data[0].len());
        let base_midi = standard_string_midi(note.string, self.instrument_type) + note.fret.max(0);
        let slide_target_midi = standard_string_midi(note.string, self.instrument_type) + note.slide_fret.max(note.fret).max(0);
        let mut phase = 0.0_f32;
        let attack = 0.01;
        let release = 0.08;

        for sample_index in start_sample..end_sample {
            let note_time = (sample_index - start_sample) as f32 / sample_rate as f32;
            let absolute_time = note.time_offset + note_time;
            let progress = (note_time / note.time_length.max(f32::EPSILON)).clamp(0.0, 1.0);
            let slide_midi = if note.has_technique(SongNoteTechnique::SLIDE) && note.slide_fret >= 0 {
                lerp(base_midi as f32, slide_target_midi as f32, progress)
            } else {
                base_midi as f32
            };
            let bend_semitones = self.get_render_bend_cents(note, absolute_time) / 100.0;
            let vibrato_semitones = if note.has_technique(SongNoteTechnique::VIBRATO) {
                (note_time * 7.0 * TAU).sin() * 0.12
            } else {
                0.0
            };
            let frequency = midi_frequency_f32(slide_midi + bend_semitones + vibrato_semitones);
            phase += frequency / sample_rate as f32;

            let envelope = note_envelope(note_time, note.time_length, attack, release);
            let mut gain = 0.10 * envelope;
            if note.has_technique(SongNoteTechnique::ACCENT) {
                gain *= 1.35;
            }
            if note.has_technique(SongNoteTechnique::PALM_MUTE) || note.has_technique(SongNoteTechnique::FRET_HAND_MUTE) {
                gain *= 0.55;
            }
            if note.has_technique(SongNoteTechnique::HARMONIC) || note.has_technique(SongNoteTechnique::PINCH_HARMONIC) {
                gain *= 0.75;
            }

            let sample = (phase * TAU).sin() * gain;
            self.sample_data[0][sample_index] += sample * 0.9;
            self.sample_data[1][sample_index] += sample * 0.9;
        }

        normalize_stereo(&mut self.sample_data);
    }

    fn render_sine_note(&mut self, frequency: f32, start_time: f32, duration: f32, gain: f32, pan: f32, sample_rate: usize) {
        let start_sample = (start_time.max(0.0) * sample_rate as f32) as usize;
        let sample_count = (duration.max(0.0) * sample_rate as f32) as usize;
        if start_sample >= self.sample_data[0].len() || sample_count == 0 {
            return;
        }

        let end_sample = (start_sample + sample_count).min(self.sample_data[0].len());
        let mut phase = 0.0_f32;
        let left_gain = gain * (1.0 - pan).clamp(0.0, 1.0);
        let right_gain = gain * (1.0 + pan).clamp(0.0, 1.0);

        for sample_index in start_sample..end_sample {
            let note_time = (sample_index - start_sample) as f32 / sample_rate as f32;
            phase += frequency / sample_rate as f32;
            let envelope = note_envelope(note_time, duration, 0.005, duration.min(0.08));
            let sample = (phase * TAU).sin() * envelope;
            self.sample_data[0][sample_index] += sample * left_gain;
            self.sample_data[1][sample_index] += sample * right_gain;
        }

        normalize_stereo(&mut self.sample_data);
    }

    fn get_render_bend_cents(&self, note: &SongNote, ref_time: f32) -> f32 {
        let Some(cents_offsets) = note.cents_offsets.as_ref() else {
            return 0.0;
        };
        let Some(first) = cents_offsets.first() else {
            return 0.0;
        };

        if ref_time <= first.time_offset {
            return first.cents as f32;
        }

        for window in cents_offsets.windows(2) {
            let start = &window[0];
            let end = &window[1];
            if ref_time <= end.time_offset {
                let duration = (end.time_offset - start.time_offset).max(f32::EPSILON);
                let progress = ((ref_time - start.time_offset) / duration).clamp(0.0, 1.0);
                return lerp(start.cents as f32, end.cents as f32, progress);
            }
        }

        cents_offsets.last().map(|offset| offset.cents as f32).unwrap_or(0.0)
    }
    
    pub fn set_playback_sample_rate(&mut self, rate: f64) {
        self.playback_sample_rate = rate;
        self.finished_playing = false;
    }
    
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed;
    }
    
    pub fn set_pitch_shift_semitones(&mut self, semitones: f64) {
        self.pitch_shift_semitones = semitones;
        self.update_pitch_shift();
    }
    
    fn update_pitch_shift(&mut self) {
        let semitones = self.tuning_offset_semitones - self.pitch_shift_semitones;
        if self.song_tuning_mode > 0 && semitones != 0.0 {
            self.pitch_shift = 1.0 / (2.0_f64).powf(semitones / 12.0);
        }
    }
    
    pub fn seek_time(&mut self, secs: f32) {
        self.seek_time = Some(secs);
        self.current_second = secs;
    }
    
    pub fn read_samples(&mut self, left_channel: &mut [f32], right_channel: &mut [f32]) {
        if let Some(seek) = self.seek_time.take() {
            if self.song_length_seconds > 0.0 && self.total_samples > 0 {
                self.current_playback_sample = ((seek / self.song_length_seconds) * self.total_samples as f32) as i64;
            }
        }
        
        if self.paused || self.finished_playing || self.total_samples <= 0 || self.sample_data.len() < 2 {
            left_channel.fill(0.0);
            right_channel.fill(0.0);
            return;
        }
        
        let samples = (left_channel.len() as i64).min(self.total_samples - self.current_playback_sample) as usize;
        
        for i in 0..samples {
            left_channel[i] = self.sample_data[0][(self.current_playback_sample as usize + i)] * self.linear_gain;
            right_channel[i] = self.sample_data[1][(self.current_playback_sample as usize + i)] * self.linear_gain;
        }
        
        for i in samples..left_channel.len() {
            left_channel[i] = 0.0;
            right_channel[i] = 0.0;
        }
        
        self.current_playback_sample += samples as i64;
        self.current_second = (self.current_playback_sample as f32 / self.total_samples as f32) * self.song_length_seconds;
        
        self.finished_playing = self.current_playback_sample >= self.total_samples;
    }
    
    pub fn get_drum_notes(&self) -> Vec<crate::song::SongDrumNote> {
        self.drum_notes.clone()
    }
    
    pub fn get_instrument_notes(&self) -> Vec<crate::song::SongNote> {
        self.instrument_notes.clone()
    }

    pub fn get_chords(&self) -> Vec<crate::song::SongChord> {
        self.chords.clone()
    }

    pub fn get_keyboard_notes(&self) -> Vec<crate::song::SongKeyboardNote> {
        self.keyboard_notes.clone()
    }

    pub fn get_song_beats(&self) -> Vec<crate::song::SongBeat> {
        self.song_structure.beats.clone()
    }
}

fn standard_string_midi(string: i32, instrument_type: SongInstrumentType) -> i32 {
    let guitar_notes = [40, 45, 50, 55, 59, 64];
    let bass_notes = [28, 33, 38, 43];
    let notes = if instrument_type == SongInstrumentType::BassGuitar {
        &bass_notes[..]
    } else {
        &guitar_notes[..]
    };

    notes[string.clamp(0, notes.len() as i32 - 1) as usize]
}

fn midi_frequency(midi_note: i32) -> f32 {
    midi_frequency_f32(midi_note as f32)
}

fn midi_frequency_f32(midi_note: f32) -> f32 {
    440.0 * 2.0_f32.powf((midi_note - 69.0) / 12.0)
}

fn note_envelope(note_time: f32, duration: f32, attack: f32, release: f32) -> f32 {
    let attack_gain = if attack > 0.0 {
        (note_time / attack).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let release_start = (duration - release).max(0.0);
    let release_gain = if note_time > release_start && release > 0.0 {
        ((duration - note_time) / release).clamp(0.0, 1.0)
    } else {
        1.0
    };

    attack_gain.min(release_gain)
}

fn normalize_stereo(sample_data: &mut [Vec<f32>]) {
    let peak = sample_data
        .iter()
        .flat_map(|channel| channel.iter())
        .fold(0.0_f32, |max_peak, sample| max_peak.max(sample.abs()));

    if peak <= 0.95 {
        return;
    }

    let scale = 0.95 / peak;
    for channel in sample_data {
        for sample in channel {
            *sample *= scale;
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn demo_note_with(
    time_offset: f32,
    time_length: f32,
    fret: i32,
    string: i32,
    hand_fret: i32,
    techniques: i32,
    chord_id: i32,
    finger_id: i32,
    slide_fret: i32,
    cents_offsets: Option<Vec<(f32, i32)>>,
) -> SongNote {
    SongNote {
        chord_id,
        time_offset,
        time_length,
        end_time: time_offset + time_length,
        fret,
        string,
        techniques,
        hand_fret,
        finger_id,
        slide_fret,
        cents_offsets: cents_offsets.map(|offsets| {
            offsets
                .into_iter()
                .map(|(offset, cents)| crate::song::CentsOffset::new(offset, cents))
                .collect()
        }),
    }
}

fn demo_drum(time_offset: f32, kit_piece: DrumKitPieceSimple) -> SongDrumNote {
    SongDrumNote {
        time_offset,
        time_length: 0.1,
        end_time: time_offset + 0.1,
        kit_piece,
        articulation: DrumArticulationSimple::DrumHead,
    }
}

fn demo_key(time_offset: f32, time_length: f32, note: i32) -> SongKeyboardNote {
    SongKeyboardNote {
        time_offset,
        time_length,
        end_time: time_offset + time_length,
        note,
    }
}

impl Default for SongPlayer {
    fn default() -> Self {
        Self::new()
    }
}
