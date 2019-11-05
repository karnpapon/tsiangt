use std::process::Command;
use std::fs::{DirEntry as StdDirEntry};
use std::fs::File;
use std::io::BufReader;
use std::cmp::Ordering;
use std::path::{ Path };
use std::fmt::{self, Formatter, Display};
use std::{ fs, io, path::PathBuf, ffi::OsStr };

use crossbeam_channel::{Receiver, Sender};

use ignore::{ Walk, DirEntry };
use rodio::{Device, Sink};
use id3::Tag;
use std::error;


use crate::player;


//#[shell]
//fn run(dir: &str) -> Result<impl Iterator<Item=String>, Box<dyn error::Error>> { r#"
//    timidity $DIR
//"# }
//



const PANEL: [&'static str; 2] =[
    "Directory",
    "Files"
];

const TABS: [&'static str; 3] = [
    "playlist",
    "library",
    "search"
];


#[derive(Debug, Clone)]
pub struct ListState<I> {
    pub items: Vec<I>,
    pub selected: usize
}

impl<I> ListState<I>{
    fn new(items: Vec<I>) -> ListState<I>{
        ListState {
            items,
            selected: 0
        } 
    }

    //TODO: make more sense to reverse next with prev. 
    fn select_next(&mut self) {
        if self.selected > 0 {
            self.selected -= 1
        }
    }

    fn select_prev( &mut self){
        if self.selected < ( self.items.len() - 1 ){
            self.selected += 1
        }
    }

    fn get_selected_item(&self) -> &I{
        &self.items[self.selected]
    }

    pub fn get_next_selected_item(&self) -> &I{
        &self.items[self.selected + 1]
    }

}

pub struct ControlState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize
}

impl<'a> ControlState<'a> {
    fn new(titles: Vec<&'a str>) -> ControlState {
       ControlState {
            titles,
            index: 0
        } 
    }

    pub fn prev_panel(&mut self){
       self.index = 0;
    }

    pub fn next_panel(&mut self){
       self.index = 1;
    }

    pub fn get_title(&self) -> &str{
        self.titles[self.index] 
    }
}

pub struct TabState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
    pub panels: ControlState<'a>
}


impl<'a> TabState<'a>{
    fn new(title: Vec<&'a str>, panels: ControlState<'a>) -> TabState<'a>{
        TabState { 
            titles: title, 
            index: 0, 
            panels
        }
    }

    pub fn get_current_title(&self) -> &str{
        self.titles[self.index % self.titles.len()]
    }

    pub fn set_tab_index(&mut self,tab: usize) {
        self.index = tab - 1; 
    }
} 

#[derive(Clone, Debug, Eq)]
pub struct Track {
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album_artist: String,
    pub album: String,
    pub year: i32,
    pub track_num: u32,
    pub duration: u32,
}

impl Track {

    pub fn new(path: PathBuf) -> Result<Track, ()> {
        let tag = Tag::read_from_path(&path);

        if tag.is_err() {
            return Err(());
        }

        let safe_tag = Tag::read_from_path(&path).unwrap();

        let mut title: String = "".to_string();
        if let Some(s) = safe_tag.title() {
            title = s.to_string();
        }

        let mut artist: String = "".to_string();
        if let Some(s) = safe_tag.artist() {
            artist = s.to_string();
        }

        let mut album: String = "".to_string();
        if let Some(s) = safe_tag.album() {
            album = s.to_string();
        }

        let album_artist;
        match safe_tag.album_artist() {
            Some(s) => {
                album_artist = s.to_string();
            }
            None => {
                album_artist = artist.clone();
            }
        }

        let mut year: i32 = 0;
        if let Some(x) = safe_tag.year() {
            year = x;
        }

        let mut track_num: u32 = 0;
        if let Some(x) = safe_tag.track() {
            track_num = x;
        }

        let mut duration: u32 = 0;
        if let Some(x) = safe_tag.duration() {
            duration = x;
        }

        Ok(Track {
            file_path: path.as_path().to_string_lossy().to_string(),
            title,
            artist,
            album_artist,
            album,
            year,
            track_num,
            duration,
        })
    }
}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Track) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Track {
    fn cmp(&self, other: &Track) -> Ordering {
        self.track_num.cmp(&other.track_num)
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Track) -> bool {
        self.file_path == other.file_path
    }
}



pub struct App<'a> {
    //pub player: Player,
    pub title: &'a str,
    pub directory: ListState<PathBuf>,
    pub directory_files: ListState<Track>,
    pub playlist: ListState<Track>,
    pub tabs: TabState<'a>,
    pub playing_track_index: Option<usize>,
    pub is_quit: bool,
    pub is_playing: bool,
    pub is_playlist_added: bool,
    pub should_select: bool,
    pub track_x: Sender<Track>,
    pub track_p_x: Sender<bool>,
    pub track_i_rx: Receiver<bool>,
    pub track_atp_x: Sender<Track>
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str, 
        track_x :Sender<Track>, 
        track_p_x: Sender<bool>, 
        track_i_rx: Receiver<bool>,
        track_atp_x: Sender<Track>
        ) -> App<'a> {
        return 
        App{
            title,
            //player,
            playlist: ListState::new(Vec::new()),
            directory: init_directory( &dirs::audio_dir().unwrap()),
            directory_files: init_tracks(&dirs::audio_dir().unwrap()),
            playing_track_index: None,
            tabs: TabState::new(TABS.to_vec(),ControlState::new(PANEL.to_vec())),
            is_quit: false,
            is_playing: false,
            is_playlist_added: false,
            should_select: false,
            track_x,
            track_p_x,
            track_i_rx,
            track_atp_x
        };
    }

    pub fn set_should_select(&mut self, state: bool){
        self.should_select = state;
    }

    pub fn on_key_up(&mut self){
        match self.tabs.get_current_title() {
           "playlist" => {self.playlist.select_next()},
           "library" => { self.handle_panel_select_next()},
           _ => {}
        }
    }

    pub fn on_key_down(&mut self) {
         match self.tabs.get_current_title() {
           "playlist" => {self.playlist.select_prev()},
           "library" => {self.handle_panel_select_prev()},
           _ => {}
        }
    }

    pub fn handle_panel_select_prev(&mut self){
        match self.tabs.panels.index {
            0 => self.directory.select_prev(),
            1 => self.directory_files.select_prev(),
            _ => {}
        }
    }


    pub fn handle_panel_select_next(&mut self){
        match self.tabs.panels.index {
            0 => self.directory.select_next(),
            1 => self.directory_files.select_next(),
            _ => {}
        }
    }

    pub fn get_playing_track_index(&self) -> Option<usize> {
        match self.tabs.index{
            0 => { Some( self.playlist.selected )},
            1 => { Some( self.directory_files.selected)},
            _ => None
        }
    }

     pub fn on_select_playing(&mut self) {
        self.is_playing = true;
        let track = self.playlist.get_selected_item().clone();
        self.track_x.send(track).unwrap();
        self.playing_track_index = self.get_playing_track_index();
    }

    pub fn set_next_queue_playing_index(&mut self){
        self.playing_track_index = Some(self.playlist.selected + 1);
    }

     pub fn on_select_directory_files_playing(&mut self){
        self.is_playlist_added = true;
        self.playlist.items.push(self.directory_files.get_selected_item().clone());
     }

     pub fn on_select_directory(&mut self){
        self.handle_get_directory();
     }

     pub fn on_select_directory_files(&mut self){
        self.handle_get_directory_files(); 
     }


    pub fn set_init_directory(&mut self, p: ListState<PathBuf>){
        self.directory = p
    }

    pub fn set_init_directory_files(&mut self, f: ListState<Track>){
        self.directory_files = f
    }

     // not such an elegant way to handle the issue, but it's working.
     pub fn get_current_item_lists(&mut self) -> usize{
        let mut size: usize = 0;
        let tab = self.tabs.get_current_title();
        if tab == "library"{
            match self.tabs.panels.get_title(){
                "Directory" => { size = self.directory.items.len()},
                "Files" => { size = self.directory_files.items.len()},
                _ => {}
            } 
        } else if tab == "playlist" {
           size = self.playlist.items.len();
        } 
        size
     }


    pub fn on_key(&mut self, c: char){
        if c.is_digit(10) { 
            self.handle_tab(c.to_digit(10).unwrap() as usize) 
        } else if c == 'q' { 
            self.is_quit = true;
        } else {

            // check if current panel has any item, 
            // otherwise disable keypress 
            // (only tab selection available).
            if self.get_current_item_lists() > 0 {
               match c {
                 '\n' => match self.tabs.get_current_title() {
                     "playlist" => self.on_select_playing(),
                     "library" => {
                         match self.tabs.panels.get_title(){
                             "Directory" => {
                                 self.on_select_directory();
                                 self.on_select_directory_files();
                             },
                             "Files" => { 
                                 self.set_should_select(true);
                                 self.on_select_directory_files_playing()
                             },
                             _ => {}
                         }
                     },
                     _ => {  }
                 },
                 'b' => match self.tabs.panels.get_title() { 
                   "Directory" => self.redirect_parent_path(),
                   _ => {}
                 },
                 ' ' => { self.is_playing = !self.is_playing; self.track_p_x.send(true).unwrap()},
                 's' => { self.is_playing = !self.is_playing; self.track_p_x.send(false).unwrap()},
                 'j' => { self.is_playlist_added = false; self.on_key_down()},
                 'k' => { self.is_playlist_added = false; self.on_key_up()},
                 'h' => { self.tabs.panels.prev_panel()},
                 'l' => { self.tabs.panels.next_panel()},
                 _ => {}
                }   
            }
        }

    }

    fn redirect_parent_path(&mut self){

        if let Some(d) = self.directory.get_selected_item().parent(){
            let d = PathBuf::from(d.parent().unwrap());

           // stop at home root folder.
           //if d !=  dirs::home_dir().unwrap(){
               let p = get_list_of_paths(&d);
               self.set_directory(p.unwrap());
           //}
        } 
        
    }


    pub fn handle_tab(&mut self, i: usize){
        self.tabs.set_tab_index(i);
    }

    pub fn handle_get_directory(&mut self){
        //TODO: handle empty_folder, display text instead?
            if let Some(res) = get_list_of_paths(self.directory.get_selected_item()){
                self.set_directory(res);
            }
    }

    pub fn handle_get_directory_files(&mut self){
        let files = get_tracks_from_path( self.directory.get_selected_item());
        if files.len() > 0 {
            self.set_directory_files(files); 
        }
    }

    pub fn set_directory(&mut self, lists: Vec<PathBuf>){
        let mut path_str = vec![];

        for p in lists {
            if is_not_hidden(&p) {
               path_str.push(p);
            }
            
        }
        self.directory =  ListState::new(path_str);
    }

    pub fn set_directory_files(&mut self, lists: Vec<Track>) {
        self.directory_files =  ListState::new(lists);
    }

   //  pub fn handle_shell(&self) ->  Result<(), Box<dyn error::Error>> {
   //     run("/Users/mac/Desktop/test.mid");
   //     Ok( () )
   //  }
   //
   //pub fn handle_shell(&self){
       // let your_command = "timidity /Users/gingliu/Desktop/test.mid";
       // let output = Command::new("bash")
       // .arg("-c").arg(your_command)
       // .output().expect("cannot spawn bash")
       // .stdout;
       // println!("{}", String::from_utf8(output).expect("Output is not utf-8"));
        //sh!("timidity /Users/gingliu/Desktop/test.mid");
   //}

}

pub fn is_not_hidden(entry: &PathBuf) -> bool {
    entry
         .file_name()
         .unwrap()
         .to_str()
         .map(|s| !s.starts_with("."))
         .unwrap_or(false)
}

fn is_music_in_folder(path: &PathBuf) -> bool {
//    let metadata = fs::metadata(entry.path()).unwrap();
//    if metadata.is_dir() {
//        return false;
//    }
//
    let mut has_audio_file: bool = false;

    for entry in path.read_dir().expect("cannot read dir"){
        if let Ok(entry) = entry {
             if let Some(extension) = entry.path().extension() {
                 match extension.to_str() {
                     Some("mp3") =>  has_audio_file = true,
                     Some("flac") => has_audio_file = true,
                     Some("ogg") => has_audio_file = true,
                     _ => return false,
                 };
             } 
         }
    }

    has_audio_file

    }

fn is_music(entry: &DirEntry) -> bool {
    let metadata = fs::metadata(entry.path()).unwrap();
    if metadata.is_dir() {
        return false;
    }

    if let Some(extension) = entry.path().extension() {
        match extension.to_str() {
            Some("mp3") => return true,
            Some("flac") => return true,
            Some("ogg") => return true,
            _ => return false,
        };
    } else {
        return false;
    }
}

pub fn init_tracks(path: &PathBuf) -> ListState<Track>{
    ListState::new(get_tracks_from_path(path).to_vec())
}


pub fn init_directory(path: &PathBuf) -> ListState<PathBuf>{
        let lists = get_list_of_paths(&path);
        let mut path_str = vec![];

        for p in lists.unwrap() {
            if is_not_hidden(&p) {
               path_str.push(p);
            }
            
        }
        ListState::new(path_str)
}




pub fn get_list_of_paths(root: &PathBuf) -> Option<Vec<PathBuf>> {
    let mut result = vec![];

    // validation.
    let is_empty_folder = fs::read_dir(root).map(|mut i| i.next().is_none()).unwrap_or(false);
    //let is_contains_no_folder = is_no_folder_inside(root);

    if  !is_empty_folder {
        for path in fs::read_dir(root).unwrap(){
            let path = path.unwrap().path();
            result.push(path.to_owned());
        }
        Some(result)
    } else {
        None
    }
    
}


fn is_no_folder_inside(d: &PathBuf) -> bool {
    let mut is_no_folder: bool = false;
    if let Ok(entries) = fs::read_dir(d) {
    for entry in entries {
        if let Ok(entry) = entry {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir(){
                    is_no_folder = false;
                } else {
                    is_no_folder = true;
                }
            } else {
                return false;
            }
        }
    }
}
    is_no_folder
}



fn get_tracks_from_path(path: &PathBuf) -> Vec<Track>{
        let mut lists = Vec::new();
        for result in Walk::new(path) {
        if let Ok(entry) = result {
            if is_music(&entry) {
                let track = Track::new(entry.into_path());
                if let Ok(t) = track{
                   lists.push(t);
                }
            }
        }
	}

        lists
}

