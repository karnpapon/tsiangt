use ignore::Walk;
use std::fs;

use std::fmt::{self, Formatter, Display};

//use shellfn::shell;
use std::error;


//#[shell]
//fn run(dir: &str) -> Result<impl Iterator<Item=String>, Box<dyn error::Error>> { r#"
//    timidity $DIR
//"# }


const LIST: [&'static str; 14]  = [
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


const POOL: [&'static str; 14] =[
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

//impl<I> Display for ListState<I> {
//    // `f` is a buffer, and this method must write the formatted string into it
//    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//        // `write!` is like `format!`, but it will write the formatted string
//        // into a buffer (the first argument)
//        write!(f, "{}", self)
//    }
//}

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

    pub fn get_tab_index(&mut self,tab: usize) {
        self.index = tab - 1; 
    }
} 

//pub struct Server<'a> {
//    pub name: &'a str,
//    pub location: &'a str,
//    pub coords: (f64, f64),
//    pub status: &'a str,
//}


pub struct App<'a> {
    pub title: &'a str,
    pub directory: ListState<&'a str>,
    pub playlist: ListState<SongData <'a> >,
    pub pools: ListState<&'a str>,
    pub tabs: TabState<'a>,
    pub is_quit: bool,
    pub current_playback: Option<()>, // might need data type.
}


impl<'a> App<'a> {
    pub fn new(title: &'a str) -> App<'a> {
        return 
        App{
            title,
            directory: ListState::new(LIST.to_vec()),
            playlist: ListState::new( SongData::new() ),
            pools: ListState::new(POOL.to_vec()),
            current_playback: None,
            tabs: TabState::new(TABS.to_vec(),ControlState::new(PANEL.to_vec())),
            is_quit: false,
        };
    }

    pub fn on_key_up(&mut self){
        match self.tabs.get_current_title() {
           "playlist" => {self.playlist.select_next()},
           "library" => match self.tabs.panels.index { 
               0 => self.directory.select_next(),
               1 => self.pools.select_next(),
               _ => {}
           } ,
           _ => {}
        }
    }

    pub fn on_key_down(&mut self) {
         match self.tabs.get_current_title() {
           "playlist" => {self.playlist.select_prev()},
           "library" => match self.tabs.panels.index { 
               0 => self.directory.select_prev(),
               1 => self.pools.select_prev(),
               _ => {}
           },
           _ => {}
        }
    }


     pub fn on_press_enter(&mut self) {
        //TODO: single active state. 
        let i = self.playlist.selected;
        match self.tabs.get_current_title(){
            "playlist" => {
                self.playlist.items[i].active = !self.playlist.items[i].active;
            },
            _ => {}
        }
        self.handle_shell();
    }


    pub fn on_key(&mut self, c: char){
        match c {
            '\n' => {self.on_press_enter();},
            'q' => { self.is_quit = true;},
            'j' => { self.on_key_down()},
            'k' => { self.on_key_up()},
            'h' => { self.tabs.panels.prev_panel()},
            'l' => { self.tabs.panels.next_panel()},
            '1' => { self.tabs.get_tab_index(1)},
            '2' => { self.tabs.get_tab_index(2)},
            '3' => { self.tabs.get_tab_index(3)},
            _ => {}
        }
    }

   
    pub fn get_path(&self){
        for result in Walk::new("./") {
   	 match result {
   	     Ok(entry) => println!("{}", entry.path().display()),
   	     Err(err) => println!("ERROR: {}", err),
   	 }
	} 
    }


   //  pub fn handle_shell(&self) ->  Result<(), Box<dyn error::Error>> {
   //     run("/Users/mac/Desktop/test.mid");
   //     Ok( () )
   //  }
   //
   pub fn handle_shell(&self){
        sh!("timidity /Users/mac/Desktop/test.mid");
   }

}


