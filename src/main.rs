#[allow(dead_code)]
mod App;
mod events;
mod ui;

use std::io;
use std::path::PathBuf;
use std::fs;


use termion::raw::IntoRawMode;
use termion::event::Key;
use tui::Terminal;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use ignore::{Walk, DirEntry};

use crate::App::App as Application;
use crate::App::*;
use crate::App::{Player, Track};
use crate::events::{ Events, Event };


#[macro_use] extern crate failure;
use failure::Error;


#[macro_use]
extern crate shells;


//fn main() -> Result<(), Box<dyn error::Error>> {
//    run("/Users/mac/Desktop/test.mid");
//    Ok( () )
//}



fn main() -> Result<(), failure::Error> {

    let events = Events::new();
    let device = rodio::default_output_device().expect("No audio output device found");
    //let track = get_tracks_from_path();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout); // important!, separated into new screen (without data overlay with standard terminal screen).
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?; // hide native terminal cursor.


    let mut audio = Player::new(device);
    let mut app = Application::new("/tsiangt/", audio);

    loop {
        ui::draw(&mut terminal, &app)?;
         match events.next()?{
             Event::Input(key) => match key {
                 Key::Char(c) => { app.on_key(c)}, 
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
             },
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





