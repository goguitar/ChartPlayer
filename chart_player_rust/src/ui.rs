//! ChartPlayer UI components

use gtk4::{Box, HeaderBar, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use gtk4::prelude::*;

/// Create main interface with header and song list
pub fn main_interface() -> gtk4::Box {
    let main_box = Box::new(Orientation::Vertical, 0);
    
    // Header bar with title
    let header = HeaderBar::new();
    let title = Label::new(Some("ChartPlayer v0.1.26"));
    header.set_title_widget(Some(&title));
    main_box.append(&header);
    
    // Song list
    let list = create_song_list();
    main_box.append(&list);
    
    main_box
}

/// Create song list display
pub fn create_song_list() -> ScrolledWindow {
    let scrolled = ScrolledWindow::new();
    scrolled.set_vexpand(true);
    
    let list = ListBox::new();
    list.set_vexpand(true);
    list.set_hexpand(true);
    
    // Header row
    let header_row = ListBoxRow::new();
    let header_box = Box::new(Orientation::Horizontal, 8);
    header_box.set_margin_start(8);
    header_box.set_margin_end(8);
    
    add_column(&header_box, "Title", 200);
    add_column(&header_box, "Artist", 150);
    add_column(&header_box, "Parts", 60);
    add_column(&header_box, "Diff", 50);
    add_column(&header_box, "Length", 60);
    add_column(&header_box, "Plays", 50);
    add_column(&header_box, "Last", 90);
    
    header_row.set_child(Some(&header_box));
    list.append(&header_row);
    
    // Sample song rows
    add_song(&list, "Test Song 1", "Artist 1", "4", "3.5", "3:45", "5", "Today");
    add_song(&list, "Test Song 2", "Artist 2", "3", "2.0", "4:12", "0", "-");
    add_song(&list, "Test Song 3", "Artist 3", "5", "5.0", "2:58", "10", "Yesterday");
    
    scrolled.set_child(Some(&list));
    scrolled
}

fn add_column(parent: &Box, text: &str, width: i32) {
    let label = Label::new(Some(text));
    label.set_width_request(width);
    label.set_halign(gtk4::Align::Start);
    parent.append(&label);
}

fn add_song(parent: &ListBox, title: &str, artist: &str, parts: &str, diff: &str, len: &str, plays: &str, last: &str) {
    let row = ListBoxRow::new();
    let row_box = Box::new(Orientation::Horizontal, 8);
    row_box.set_margin_start(8);
    row_box.set_margin_end(8);
    
    add_column(&row_box, title, 200);
    add_column(&row_box, artist, 150);
    add_column(&row_box, parts, 60);
    add_column(&row_box, diff, 50);
    add_column(&row_box, len, 60);
    add_column(&row_box, plays, 50);
    add_column(&row_box, last, 90);
    
    row.set_child(Some(&row_box));
    parent.append(&row);
}