//! ChartPlayer UI components with playback controls

use gtk4::{Box, Button, HeaderBar, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use gtk4::prelude::*;

pub fn main_interface() -> gtk4::Box {
    let main_box = Box::new(Orientation::Vertical, 0);
    
    let header = HeaderBar::new();
    let title = Label::new(Some("ChartPlayer v0.1.26"));
    header.set_title_widget(Some(&title));
    main_box.append(&header);
    
    let controls = create_playback_controls();
    main_box.append(&controls);
    
    let list = create_song_list();
    main_box.append(&list);
    
    main_box
}

pub fn create_playback_controls() -> Box {
    let controls = Box::new(Orientation::Horizontal, 8);
    controls.set_margin_start(8);
    controls.set_margin_end(8);
    controls.set_margin_top(4);
    controls.set_margin_bottom(4);
    
    // Previous
    let prev = Button::new();
    prev.set_icon_name("media-skip-backward-symbolic");
    prev.set_tooltip_text(Some("Previous"));
    controls.append(&prev);
    
    // Play
    let play = Button::new();
    play.set_icon_name("media-playback-start-symbolic");
    play.set_tooltip_text(Some("Play"));
    controls.append(&play);
    
    // Stop
    let stop = Button::new();
    stop.set_icon_name("media-playback-stop-symbolic");
    stop.set_tooltip_text(Some("Stop"));
    controls.append(&stop);
    
    // Next
    let next = Button::new();
    next.set_icon_name("media-skip-forward-symbolic");
    next.set_tooltip_text(Some("Next"));
    controls.append(&next);
    
    // Time
    let time = Label::new(Some("0:00 / 0:00"));
    time.set_width_request(100);
    controls.append(&time);
    
    // Progress placeholder
    let progress = Label::new(Some("======================"));
    progress.set_hexpand(true);
    controls.append(&progress);
    
    // Speed
    let speed = Button::new();
    speed.set_label("100%");
    speed.set_tooltip_text(Some("Speed"));
    controls.append(&speed);
    
    // Volume
    let volume = Button::new();
    volume.set_icon_name("audio-volume-high-symbolic");
    volume.set_tooltip_text(Some("Volume"));
    controls.append(&volume);
    
    controls
}

pub fn create_song_list() -> ScrolledWindow {
    let scrolled = ScrolledWindow::new();
    scrolled.set_vexpand(true);
    
    let list = ListBox::new();
    list.set_vexpand(true);
    list.set_hexpand(true);
    
    // Header
    let header_row = ListBoxRow::new();
    let header_box = Box::new(Orientation::Horizontal, 8);
    header_box.set_margin_start(8);
    header_box.set_margin_end(8);
    
    col(&header_box, "Title", 200);
    col(&header_box, "Artist", 150);
    col(&header_box, "Parts", 60);
    col(&header_box, "Diff", 50);
    col(&header_box, "Length", 60);
    col(&header_box, "Plays", 50);
    col(&header_box, "Last", 90);
    
    header_row.set_child(Some(&header_box));
    list.append(&header_row);
    
    // Sample rows
    row(&list, "Test Song 1", "Artist 1", "4", "3.5", "3:45", "5", "Today");
    row(&list, "Test Song 2", "Artist 2", "3", "2.0", "4:12", "0", "-");
    row(&list, "Test Song 3", "Artist 3", "5", "5.0", "2:58", "10", "Yesterday");
    
    scrolled.set_child(Some(&list));
    scrolled
}

fn col(parent: &Box, text: &str, width: i32) {
    let label = Label::new(Some(text));
    label.set_width_request(width);
    label.set_halign(gtk4::Align::Start);
    parent.append(&label);
}

fn row(parent: &ListBox, title: &str, artist: &str, parts: &str, diff: &str, len: &str, plays: &str, last: &str) {
    let row = ListBoxRow::new();
    let row_box = Box::new(Orientation::Horizontal, 8);
    row_box.set_margin_start(8);
    row_box.set_margin_end(8);
    
    col(&row_box, title, 200);
    col(&row_box, artist, 150);
    col(&row_box, parts, 60);
    col(&row_box, diff, 50);
    col(&row_box, len, 60);
    col(&row_box, plays, 50);
    col(&row_box, last, 90);
    
    row.set_child(Some(&row_box));
    parent.append(&row);
}