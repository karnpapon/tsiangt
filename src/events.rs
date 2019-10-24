use std::io;
use std::sync::mpsc;
use std::thread;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I>{
    Input(I)
}

pub struct Events {
    pub rcvs: mpsc::Receiver<Event<Key>>,
    pub input_handle: thread::JoinHandle<()>
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub exit_key: Key
}


impl Default for Config{
    fn default() -> Config {
        return Config {
            exit_key: Key::Char('q') 
        } 
    }
}


impl Events {

    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    match evt {
                        Ok(key) => {
                            if let Err(_) = tx.send(Event::Input(key)) {
                                return;
                            }
                            if key == config.exit_key {
                                return;
                            }
                        }
                        Err(_) => {}
                    }
                }
            })
        };
        
        Events {
            rcvs: rx,
            input_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rcvs.recv()
    }

}
