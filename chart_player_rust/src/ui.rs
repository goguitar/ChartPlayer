//! ChartPlayer UI - matching original look

use gtk4::{Box,Button,HeaderBar,Label,ListBox,ListBoxRow,Orientation,ScrolledWindow};
use gtk4::prelude::*;
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
use std::sync::Arc;

pub struct Player{
    pub is_playing:AtomicBool,
    pub paused:AtomicBool,
    pub total_seconds:f64,
    pub current_song:AtomicUsize,
}

impl Default for Player{
    fn default()->Self{
        Self{
            is_playing:AtomicBool::new(false),
            paused:AtomicBool::new(true),
            total_seconds:225.0,
            current_song:AtomicUsize::new(0),
        }
    }
}

impl Player{
    pub fn toggle_play(&self){
        if self.paused.load(Ordering::SeqCst){
            self.paused.store(false,Ordering::SeqCst);
            self.is_playing.store(true,Ordering::SeqCst);
        }else{
            self.paused.store(true,Ordering::SeqCst);
        }
    }
    pub fn stop(&self){
        self.paused.store(true,Ordering::SeqCst);
        self.is_playing.store(false,Ordering::SeqCst);
    }
    pub fn next(&self){
        let c=self.current_song.load(Ordering::SeqCst);
        if c<2{self.current_song.store(c+1,Ordering::SeqCst);}
    }
    pub fn prev(&self){
        let c=self.current_song.load(Ordering::SeqCst);
        if c>0{self.current_song.store(c-1,Ordering::SeqCst);}
    }
    pub fn is_paused(&self)->bool{self.paused.load(Ordering::SeqCst)}
}

pub fn main_interface(player: Arc<std::sync::Mutex<Player>>) -> gtk4::Box {
    let mb = Box::new(Orientation::Vertical, 0);
    
    let h = HeaderBar::new();
    let t = Label::new(Some("ChartPlayer v0.1.26"));
    h.set_title_widget(Some(&t));
    mb.append(&h);
    
    mb.append(&create_controls(player.clone()));
    mb.append(&create_menu_buttons());
    mb.append(&create_song_list());
    
    mb
}

fn create_controls(player: Arc<std::sync::Mutex<Player>>) -> Box {
    let c = Box::new(Orientation::Horizontal, 4);
    c.set_margin_start(4);c.set_margin_end(4);
    c.set_margin_top(2);c.set_margin_bottom(2);
    
    let rewind = Button::new();
    rewind.set_icon_name("media-seek-backward-symbolic");
    let p1 = player.clone();
    rewind.connect_clicked(move |_|{if let Ok(pl)=p1.lock(){pl.prev();}});
    c.append(&rewind);
    
    let play = Button::new();
    play.set_icon_name("media-playback-start-symbolic");
    let p2 = player.clone();
    let bp = play.clone();
    play.connect_clicked(move |_|{
        if let Ok(pl)=p2.lock(){
            pl.toggle_play();
            let ico = if pl.is_paused(){"media-playback-start-symbolic"}else{"media-playback-pause-symbolic"};
            bp.set_icon_name(ico);
        }
    });
    c.append(&play);
    
    let stop = Button::new();
    stop.set_icon_name("media-playback-stop-symbolic");
    let p3 = player.clone();
    stop.connect_clicked(move |_|{if let Ok(pl)=p3.lock(){pl.stop();}});
    c.append(&stop);
    
    let tm = Label::new(Some("0:00"));
    tm.set_width_request(80);
    c.append(&tm);
    
    let pg = Label::new(Some("========================="));
    pg.set_hexpand(true);
    c.append(&pg);
    
    let total = Label::new(Some("/ 3:45"));
    c.append(&total);
    
    c
}

fn create_menu_buttons() -> Box {
    let m = Box::new(Orientation::Horizontal, 2);
    m.set_margin_start(4);m.set_margin_end(4);
    
    let songs = Button::new();
    songs.set_label("Songs");
    m.append(&songs);
    
    let opts = Button::new();
    opts.set_label("Options");
    m.append(&opts);
    
    let help = Button::new();
    help.set_label("?");
    m.append(&help);
    
    let notes = Button::new();
    notes.set_label("Notes");
    m.append(&notes);
    
    let tuner = Button::new();
    tuner.set_label("Tuner");
    m.append(&tuner);
    
    let sp = Label::new(Some(""));
    sp.set_hexpand(true);
    m.append(&sp);
    
    let speed = Button::new();
    speed.set_label("100%");
    m.append(&speed);
    
    let tun = Button::new();
    tun.set_label("-");
    m.append(&tun);
    
    m
}

pub fn create_song_list() -> ScrolledWindow {
    let sc = ScrolledWindow::new();
    sc.set_vexpand(true);
    let lb = ListBox::new();
    lb.set_vexpand(true);lb.set_hexpand(true);
    lb.set_selection_mode(gtk4::SelectionMode::Single);
    
    let hr = ListBoxRow::new();
    let hb = Box::new(Orientation::Horizontal,4);
    hb.set_margin_start(4);hb.set_margin_end(4);
    hb.append(&mk_lbl("Title",200));
    hb.append(&mk_lbl("Artist",150));
    hb.append(&mk_lbl("Parts",60));
    hb.append(&mk_lbl("Diff",50));
    hb.append(&mk_lbl("Length",60));
    hb.append(&mk_lbl("Plays",50));
    hb.append(&mk_lbl("Last",90));
    hr.set_child(Some(&hb));
    lb.append(&hr);
    
    add_song(&lb,"Test Song 1","Artist 1","4","3.5","3:45","5","Today");
    add_song(&lb,"Test Song 2","Artist 2","3","2.0","4:12","0","-");
    add_song(&lb,"Test Song 3","Artist 3","5","5.0","2:58","10","Yesterday");
    
    sc.set_child(Some(&lb));
    sc
}

fn mk_lbl(t:&str,w:i32)->Label{
    let l=Label::new(Some(t));l.set_width_request(w);l.set_halign(gtk4::Align::Start);l
}

fn add_song(p:&ListBox,t:&str,a:&str,pt:&str,df:&str,ln:&str,pl:&str,ls:&str){
    let r=ListBoxRow::new();
    let b=Box::new(Orientation::Horizontal,4);
    b.set_margin_start(4);b.set_margin_end(4);
    b.append(&mk_lbl(t,200));
    b.append(&mk_lbl(a,150));
    b.append(&mk_lbl(pt,60));
    b.append(&mk_lbl(df,50));
    b.append(&mk_lbl(ln,60));
    b.append(&mk_lbl(pl,50));
    b.append(&mk_lbl(ls,90));
    r.set_child(Some(&b));
    p.append(&r);
}