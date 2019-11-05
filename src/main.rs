#[allow(dead_code)]
mod App;
mod events;
mod ui;
mod custom_widgets;
mod player;

use std::io;
use std::path::PathBuf;
use std::fs;
use std::time::Duration;

extern crate dirs;

use termion::raw::IntoRawMode;
use termion::event::Key;
use tui::Terminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use ignore::{Walk, DirEntry};

use crate::App::App as Application;
use crate::App::*;
use crate::App::{Track};
use crate::events::{ Events, Event };
use crate::player::{ Player };

use std::thread;
use crossbeam_channel as channel;


use clap::{clap_app, crate_version};


#[macro_use] extern crate failure;
use failure::Error;


//#[macro_use]
//extern crate shells;


//fn main() -> Result<(), Box<dyn error::Error>> {
//    run("/Users/mac/Desktop/test.mid");
//    Ok( () )
//}



fn main() -> Result<(), failure::Error> {

    let clap = clap_app!( tsiangt =>
                          (version:crate_version!())
                          (author:"Karnpapon Boonput")
                          (about:"tsiangt terminal music player!")
                          (@arg directory: -d +takes_value "Sets directory")
    )
    .get_matches();

    let handle_events = Events::new();
    let device = rodio::default_output_device().expect("No audio output device found");

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout); // important!, separated into new screen (without data overlay with standard terminal screen).
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?; // hide native terminal cursor.
    terminal.clear()?;


    let (track_x, track_rx) = channel::bounded(0); // Track
    let (track_p_x, track_p_rx) = channel::bounded(0); // Track's play/pause.
    let (track_i_x, track_i_rx) = channel::bounded(0); // Track's information (when to start/stop).
    let (track_atp_x, track_atp_rx) = channel::bounded(0); // Track's autoplay.

    let mut app = Application::new("/tsiangt/", track_x, track_p_x, track_i_rx, track_atp_x);  
    let mut audio = Player::new(device, track_rx, track_p_rx, track_i_x, track_atp_rx);

    match clap.value_of("directory"){
        Some(c) => {
            let d = PathBuf::from(c);
            &app.set_init_directory(init_directory(&d));
            &app.set_init_directory_files(init_tracks(&d));
        },
        _ => {}
    };

    thread::spawn(move|| {
        loop{

            if audio.handler.empty(){
                if let Ok(()) = audio.track_i_x.send_timeout(true, Duration::from_millis(250)){}
            } else if let Ok(()) = audio.track_i_x.send_timeout(false, Duration::from_millis(250)){
            }

            if let Ok(track) = audio.track_rx.try_recv() {
                audio.play(track)
            }


            if let Ok(next_in_queue) = audio.track_atp_rx.try_recv(){
                audio.play(next_in_queue);
            }

            match audio.track_p_rx.try_recv(){
                Ok(true) => audio.pause(),
                Ok(false) => audio.stop(),
                _ => {}
            }
        }
    });
     
    loop {
        ui::draw(&mut terminal, &app)?;
        if let Event::Input(input) = handle_events.next()? {
         match input {
                 Key::Char(c) => { app.on_key(c)}, 
                 Key::Esc => { app.is_quit = true },
                 Key::Up => { app.on_key_up() },
                 Key::Down => { app.on_key_down();},
                 Key::Left => match app.tabs.get_current_title(){ 
                     "playlist" => {},
                     "library" => app.tabs.panels.prev_panel(),
                     _ => {}
                 },
                 Key::Right => match app.tabs.get_current_title(){ 
                     "playlist" => {},
                     "library" => app.tabs.panels.next_panel(),
                     _ => {}
                 },
                 _ => {}
             }
         }

               
        if let Ok(true) = app.track_i_rx.recv_timeout(Duration::from_millis(250)){
            if app.playlist.items.len() > 1 && app.is_playing{
                if let Ok(()) = app.track_atp_x.send_timeout(
                    app.playlist.get_next_selected_item().clone(), 
                    Duration::from_millis(250)
                ){};
                app.set_next_queue_playing_index();
            }
        } 

         if app.is_quit {
              break;
         }
    }
        Ok( () )
   
}


//#[shell]
//fn list_modified(dir: &str) -> Result<impl Iterator<Item=String>, Box<error::Error>> { r#"
//    cd $DIR
//    git status | grep '^\s*modified:' | awk '{print $2}'
//"# }





