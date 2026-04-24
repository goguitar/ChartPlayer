//! ChartPlayer UI - Player struct

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct Player{
    pub is_playing: AtomicBool,
    pub paused: AtomicBool,
    pub total_seconds: f64,
    pub current_song: AtomicUsize,
}

impl Default for Player{
    fn default() -> Self{
        Self{
            is_playing: AtomicBool::new(false),
            paused: AtomicBool::new(true),
            total_seconds: 225.0,
            current_song: AtomicUsize::new(0),
        }
    }
}

impl Player{
    pub fn toggle_play(&self){
        if self.paused.load(Ordering::SeqCst){
            self.paused.store(false, Ordering::SeqCst);
            self.is_playing.store(true, Ordering::SeqCst);
        } else {
            self.paused.store(true, Ordering::SeqCst);
        }
    }
    pub fn stop(&self){
        self.paused.store(true, Ordering::SeqCst);
        self.is_playing.store(false, Ordering::SeqCst);
    }
    pub fn next(&self){
        let c = self.current_song.load(Ordering::SeqCst);
        if c < 2 { self.current_song.store(c + 1, Ordering::SeqCst); }
    }
    pub fn prev(&self){
        let c = self.current_song.load(Ordering::SeqCst);
        if c > 0 { self.current_song.store(c - 1, Ordering::SeqCst); }
    }
    pub fn is_paused(&self) -> bool{ self.paused.load(Ordering::SeqCst) }
    pub fn is_playing(&self) -> bool{ self.is_playing.load(Ordering::SeqCst) }
}