//! UI with button signals

use gtk4::{Box,Button,HeaderBar,Label,ListBox,ListBoxRow,Orientation,ScrolledWindow};
use gtk4::prelude::*;
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
use std::sync::Arc;

pub struct Player{
    pub is_playing:AtomicBool,
    pub paused:AtomicBool,
    pub total_seconds:f64,
    pub speed_percent:f64,
    pub volume_percent:f64,
    pub current_song:AtomicUsize,
}

impl Default for Player{
    fn default()->Self{
        Self{
            is_playing:AtomicBool::new(false),
            paused:AtomicBool::new(true),
            total_seconds:225.0,
            speed_percent:100.0,
            volume_percent:100.0,
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
    mb.append(&create_playback_controls(player.clone()));
    mb.append(&create_song_list());
    mb
}

pub fn create_playback_controls(player: Arc<std::sync::Mutex<Player>>) -> Box {
    let c = Box::new(Orientation::Horizontal, 8);
    c.set_margin_start(8);c.set_margin_end(8);
    c.set_margin_top(4);c.set_margin_bottom(4);
    
    let prev = Button::new();
    prev.set_icon_name("media-skip-backward-symbolic");
    prev.set_tooltip_text(Some("Previous"));
    let p1 = player.clone();
    prev.connect_clicked(move |_btn|{
        if let Ok(pl)=p1.lock(){pl.prev();}
    });
    c.append(&prev);
    
    let play = Button::new();
    play.set_icon_name("media-playback-start-symbolic");
    play.set_tooltip_text(Some("Play"));
    let p2 = player.clone();
    let bplay = play.clone();
    play.connect_clicked(move |_btn|{
        if let Ok(pl)=p2.lock(){
            pl.toggle_play();
            let ico = if pl.is_paused(){"media-playback-start-symbolic"}else{"media-playback-pause-symbolic"};
            bplay.set_icon_name(ico);
        }
    });
    c.append(&play);
    
    let stop = Button::new();
    stop.set_icon_name("media-playback-stop-symbolic");
    stop.set_tooltip_text(Some("Stop"));
    let p3 = player.clone();
    stop.connect_clicked(move |_btn|{
        if let Ok(pl)=p3.lock(){pl.stop();}
    });
    c.append(&stop);
    
    let next = Button::new();
    next.set_icon_name("media-skip-forward-symbolic");
    next.set_tooltip_text(Some("Next"));
    let p4 = player.clone();
    next.connect_clicked(move |_btn|{
        if let Ok(pl)=p4.lock(){pl.next();}
    });
    c.append(&next);
    
    let tm = Label::new(Some("0:00 / 3:45"));
    tm.set_width_request(100);
    c.append(&tm);
    
    let pg = Label::new(Some("======================"));
    pg.set_hexpand(true);
    c.append(&pg);
    
    let sp = Button::new();
    sp.set_label("100%");
    sp.set_tooltip_text(Some("Speed"));
    c.append(&sp);
    
    let vl = Button::new();
    vl.set_icon_name("audio-volume-high-symbolic");
    vl.set_tooltip_text(Some("Volume"));
    c.append(&vl);
    
    c
}

pub fn create_song_list() -> ScrolledWindow {
    let sc = ScrolledWindow::new();
    sc.set_vexpand(true);
    let lb = ListBox::new();
    lb.set_vexpand(true);lb.set_hexpand(true);
    let hr = ListBoxRow::new();
    let hb = Box::new(Orientation::Horizontal,8);
    hb.set_margin_start(8);hb.set_margin_end(8);
    col(&hb,"Title",200);col(&hb,"Artist",150);col(&hb,"Parts",60);
    col(&hb,"Diff",50);col(&hb,"Length",60);col(&hb,"Plays",50);col(&hb,"Last",90);
    hr.set_child(Some(&hb));lb.append(&hr);
    row(&lb,"Test Song 1","Artist 1","4","3.5","3:45","5","Today");
    row(&lb,"Test Song 2","Artist 2","3","2.0","4:12","0","-");
    row(&lb,"Test Song 3","Artist 3","5","5.0","2:58","10","Yesterday");
    sc.set_child(Some(&lb));sc
}

fn col(p:&Box,t:&str,w:i32){
    let l=Label::new(Some(t));l.set_width_request(w);l.set_halign(gtk4::Align::Start);p.append(&l);
}

fn row(p:&ListBox,t:&str,a:&str,pt:&str,df:&str,ln:&str,pl:&str,ls:&str){
    let r=ListBoxRow::new();let b=Box::new(Orientation::Horizontal,8);
    b.set_margin_start(8);b.set_margin_end(8);
    col(&b,t,200);col(&b,a,150);col(&b,pt,60);col(&b,df,50);col(&b,ln,60);col(&b,pl,50);col(&b,ls,90);
    r.set_child(Some(&b));p.append(&r);
}