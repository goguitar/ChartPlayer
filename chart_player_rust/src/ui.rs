//! UI

use gtk4::{Box,Button,HeaderBar,Label,ListBox,ListBoxRow,Orientation,ScrolledWindow};
use gtk4::prelude::*;
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};

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

pub fn main_interface()->gtk4::Box{
    let mb=Box::new(Orientation::Vertical,0);
    let h=HeaderBar::new();let t=Label::new(Some("ChartPlayer v0.1.26"));h.set_title_widget(Some(&t));
    mb.append(&h);mb.append(&create_controls());mb.append(&create_list());mb
}

pub fn create_controls()->Box{
    let c=Box::new(Orientation::Horizontal,8);
    c.set_margin_start(8);c.set_margin_end(8);c.set_margin_top(4);c.set_margin_bottom(4);
    let pb=Button::new();pb.set_icon_name("media-skip-backward-symbolic");pb.set_tooltip_text(Some("Previous"));c.append(&pb);
    let py=Button::new();py.set_icon_name("media-playback-start-symbolic");py.set_tooltip_text(Some("Play"));c.append(&py);
    let st=Button::new();st.set_icon_name("media-playback-stop-symbolic");st.set_tooltip_text(Some("Stop"));c.append(&st);
    let nx=Button::new();nx.set_icon_name("media-skip-forward-symbolic");nx.set_tooltip_text(Some("Next"));c.append(&nx);
    let tm=Label::new(Some("0:00"));tm.set_width_request(100);c.append(&tm);
    let pg=Label::new(Some("======"));pg.set_hexpand(true);c.append(&pg);
    let sp=Button::new();sp.set_label("100%");sp.set_tooltip_text(Some("Speed"));c.append(&sp);
    let vl=Button::new();vl.set_icon_name("audio-volume-high-symbolic");vl.set_tooltip_text(Some("Volume"));c.append(&vl);
    c
}

pub fn create_list()->ScrolledWindow{
    let sc=ScrolledWindow::new();sc.set_vexpand(true);
    let lb=ListBox::new();lb.set_vexpand(true);lb.set_hexpand(true);
    let hr=ListBoxRow::new();let hb=Box::new(Orientation::Horizontal,8);hb.set_margin_start(8);hb.set_margin_end(8);
    col(&hb,"Title",200);col(&hb,"Artist",150);col(&hb,"Parts",60);col(&hb,"Diff",50);col(&hb,"Length",60);col(&hb,"Plays",50);col(&hb,"Last",90);
    hr.set_child(Some(&hb));lb.append(&hr);
    row(&lb,"Test Song 1","Artist 1","4","3.5","3:45","5","Today");
    row(&lb,"Test Song 2","Artist 2","3","2.0","4:12","0","-");
    row(&lb,"Test Song 3","Artist 3","5","5.0","2:58","10","Yesterday");
    sc.set_child(Some(&lb));sc
}

fn col(p:&Box,t:&str,w:i32){let l=Label::new(Some(t));l.set_width_request(w);l.set_halign(gtk4::Align::Start);p.append(&l);}
fn row(p:&ListBox,t:&str,a:&str,pt:&str,df:&str,ln:&str,pl:&str,ls:&str){
    let r=ListBoxRow::new();let b=Box::new(Orientation::Horizontal,8);b.set_margin_start(8);b.set_margin_end(8);
    col(&b,t,200);col(&b,a,150);col(&b,pt,60);col(&b,df,50);col(&b,ln,60);col(&b,pl,50);col(&b,ls,90);
    r.set_child(Some(&b));p.append(&r);
}