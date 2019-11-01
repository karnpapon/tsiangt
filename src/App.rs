use std::process::Command;
use std::fs::{DirEntry as StdDirEntry};
use std::fs::File;
use std::io::BufReader;
use std::cmp::Ordering;
use std::path::{ Path };
use std::fmt::{self, Formatter, Display};
use std::{ fs, io, path::PathBuf, ffi::OsStr };

use ignore::{ Walk, DirEntry };
use rodio::{Device, Sink};
use id3::Tag;
use std::error;


//#[shell]
//fn run(dir: &str) -> Result<impl Iterator<Item=String>, Box<dyn error::Error>> { r#"
//    timidity $DIR
//"# }
//


const LIST: [&'static str; 28]  = [
    "/folder01",
    "/folder02",
    "/folder03",
    "/folder04",
    "/folder05",
    "/music",
    "/relax",
    "/misc",
    "/hd",
    "/coding",
    "/foo",
    "/bar",
    "/folder01",
    "/folder02",
    "/folder03",
    "/folder04",
    "/folder05",
    "/music",
    "/relax",
    "/misc",
    "/hd",
    "/coding",
    "/foo",
    "/bar",
    "/baz",
    "/enry",
    "/baz",
    "/enry"
];


const PLAYLIST: [&'static str; 5] = [
    "warp portals",
    "paper",
    "gingliu",
    "karnpapon",
    "lagoon"
];


const PANEL: [&'static str; 2] =[
    "Directory",
    "Files"
];

const TABS: [&'static str; 3] = [
    "playlist",
    "library",
    "search"
];


const FILES: [&'static str; 14] =[
    "selected song1",
    "selected song2",
    "selected song3",
    "selected song4",
    "selected song5",
    "selected song6",
    "selected song7",
    "selected song8",
    "selected song9",
    "selected song1q",
    "selected song12",
    "selected song13",
    "selected song14",
    "selected song15",
];

#[derive(Clone,Copy,Debug)]
pub struct SongData<'a> {
    pub title: &'a str,
    pub artist: &'a str,
    pub album: &'a str,
    pub length: &'a str,
    pub path: &'a str,
    pub active: bool,
    pub id: usize
}


// TODO: needs dynamic data.
impl<'a> SongData<'a> {
    fn new() -> Vec<SongData<'a>>{
        MOCKDATA.to_vec()
    }
}

const MOCKDATA: [SongData; 5] = [
        SongData{
            title: "song selected 1",
            artist: "dkjjj",
            album: "oiiii",
            length: "12.34",
            path: "/Users/mac/Desktop/test.mid",
            active: false,
            id: 1
        },
        SongData{
            title: "oiuiou",
            artist: "fjskldfj",
            album: "XXXXxxx",
            length: "12.34",
            path: "/Users/mac/Desktop/test.mid",
            active: false,
            id: 2
        },
        SongData{
            title: "lagooon",
            artist: "i1o1ieu",
            album: "1209fd",
            length: "12.34",
            path: "/Users/mac/Desktop/test.mid",
            active: false,
            id: 3
        },
        SongData{
            title: "oraora",
            artist: "ffififif",
            album: "ippo",
            length: "12.34",
            path: "/Users/mac/Desktop/test.mid",
            active: false,
            id: 4
        },
        SongData{
            title: "hhhhhhhhhh",
            artist: "thehththe",
            album: "oweiei",
            length: "12.34",
            path: "/Users/mac/Desktop/test.mid",
            active: false,
            id: 5
        }
    ];



pub struct DirectoryList{
    pub path: PathBuf,
    pub name: String,
    pub index: usize
}


impl DirectoryList {

pub fn new(&mut self, p: PathBuf) -> DirectoryList{
    DirectoryList{
        path: p,
        name: String::from("/directory00"),
        index: 0
    }
}
}

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

        // TODO: check whether using hardcoded index or dynamic.
       // if self.index < ( self.titles.len() - 1 ) {
       //     self.index += 1
       // } 
       self.index = 0;
    }

    pub fn next_panel(&mut self){
        
        // TODO: check whether using hardcoded index or dynamic.
       // if self.index > 0 {
       //     self.index -= 1 
       // } 
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


pub struct Player{
   pub device: Device, 
   pub handler: Sink
}

impl Player {
    pub fn new(d: Device) -> Player {
        Player{
            handler: Sink::new(&d),
            device: d
        }
    }

    pub fn play(&mut self, track: Track){
        self.handler = Sink::new(&self.device);
        let file = File::open(&track.file_path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        self.handler.append(source);
    }

    pub fn pause(&mut self){
        if self.handler.is_paused() {
            self.handler.play();
        } else {
            self.handler.pause();
        }
    }

    pub fn stop(&mut self) {
        self.handler = Sink::new(&self.device);
    }
}


pub struct App<'a> {
    pub player: Player,
    pub title: &'a str,
    pub directory: ListState<PathBuf>,
    pub directory_files: ListState<Track>,
    pub playlist: ListState<Track>,
    pub tabs: TabState<'a>,
    pub is_quit: bool,
    pub is_playing: bool,
    pub current_playback: Option<()>, // might need data type.
    pub playing_track_index: Option<usize>,
    pub is_playlist_added: bool
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, player: Player) -> App<'a> {
        return 
        App{
            title,
            player,
            playlist: ListState::new(Vec::new()),
            directory: init_directory( &dirs::audio_dir().unwrap()),
            directory_files: init_tracks(&dirs::audio_dir().unwrap()),
            current_playback: None,
            playing_track_index: None,
            tabs: TabState::new(TABS.to_vec(),ControlState::new(PANEL.to_vec())),
            is_quit: false,
            is_playing: false,
            is_playlist_added: false
        };
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
        let i = self.playlist.selected;

        self.player.play(self.playlist.items[i].clone());
        self.playing_track_index = self.get_playing_track_index();
    }

     pub fn on_select_directory_files_playing(&mut self){
        self.is_playlist_added = true;
        let i = self.directory_files.selected;
        self.playlist.items.push(self.directory_files.items[i].clone());
     }

     pub fn on_select_directory(&mut self){
        self.handle_get_directory();
     }

     pub fn on_select_directory_files(&mut self){
        self.handle_get_directory_files(); 
     }


    pub fn on_key(&mut self, c: char){
        if c.is_digit(10) { 
            self.handle_tab(c.to_digit(10).unwrap() as usize) 
        } else { 
            match c {
            '\n' => match self.tabs.get_current_title() {
                "playlist" => self.on_select_playing(),
                "library" => {
                    match self.tabs.panels.get_title(){
                        "Directory" => {
                            self.on_select_directory();
                            self.on_select_directory_files();
                        },
                        "Files" => { self.on_select_directory_files_playing()},
                        _ => {}
                    }
                },
                _ => {  }
            },
            'b' => match self.tabs.panels.get_title() { 
              "Directory" => self.redirect_parent_path(),
              _ => {}
            },
            ' ' => { self.player.pause()},
            'q' => { self.is_quit = true;},
            'j' => { self.is_playlist_added = false; self.on_key_down()},
            'k' => { self.is_playlist_added = false; self.on_key_up()},
            'h' => { self.tabs.panels.prev_panel()},
            'l' => { self.tabs.panels.next_panel()},
            _ => {}
        }
        }

    }

    fn redirect_parent_path(&mut self){

        if let Some(d) = &self.directory.items[self.directory.selected].parent(){
            let d = PathBuf::from(d.parent().unwrap());

           // stop at home root folder.
           if d !=  dirs::home_dir().unwrap(){
               let p = get_list_of_paths(&d);
               self.set_directory(p.unwrap());
           }
        } 
        
    }


    pub fn handle_tab(&mut self, i: usize){
        self.tabs.set_tab_index(i);
    }

    pub fn handle_get_directory(&mut self){
        //TODO: handle empty_folder, display text instead?
            let item = &self.directory.items;
            let i = self.directory.selected;
            if let Some(res) = get_list_of_paths(&item[i]){
                self.set_directory(res);
            }
    }

    pub fn handle_get_directory_files(&mut self){
        let files = get_tracks_from_path( &self.directory.items[self.directory.selected]);
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

    pub fn get_directory_files(&mut self){
        let p = self.directory.items[self.directory.selected].clone();
        let d = get_tracks_from_path(&p);
        //self.directory_files = ListState::new(d);
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

fn is_not_hidden(entry: &PathBuf) -> bool {
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

fn init_tracks(path: &PathBuf) -> ListState<Track>{
    ListState::new(get_tracks_from_path(path).to_vec())
}

fn init_directory(path: &PathBuf) -> ListState<PathBuf>{
        let lists = get_list_of_paths(&path);
        let mut path_str = vec![];

        for p in lists.unwrap() {
            if is_not_hidden(&p) {
               path_str.push(p);
            }
            
        }
        ListState::new(path_str)
}


fn get_list_of_paths(root: &PathBuf) -> Option<Vec<PathBuf>> {
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

