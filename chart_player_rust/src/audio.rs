//! Audio module for song playback
//! 
//! Provides audio playback, resampling, and mixing functionality.

use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
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
    
    pub fn set_playback_sample_rate(&mut self, rate: f64) {
        self.playback_sample_rate = rate;
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
            self.current_playback_sample = ((seek / self.song_length_seconds) * self.total_samples as f32) as i64;
        }
        
        if self.paused || self.finished_playing {
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
        self.current_second = ((self.current_playback_sample as f32 / self.total_samples as f32) * self.song_length_seconds);
        
        self.finished_playing = self.current_playback_sample >= self.total_samples;
    }
    
    pub fn get_drum_notes(&self) -> Vec<crate::song::SongDrumNote> {
        Vec::new()
    }
    
    pub fn get_instrument_notes(&self) -> Vec<crate::song::SongNote> {
        Vec::new()
    }
}

impl Default for SongPlayer {
    fn default() -> Self {
        Self::new()
    }
}