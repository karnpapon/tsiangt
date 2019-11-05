use rodio::{Device, Sink};
use crate::App::{ Track};
use std::fs::File;
use std::io::BufReader;
use crossbeam_channel::{Receiver, Sender};

pub struct Player{
   pub device: Device, 
   pub handler: Sink,
   pub track_rx: Receiver<Track>,
   pub track_p_rx: Receiver<bool>,
   pub track_i_x: Sender<bool>,
   pub track_atp_rx: Receiver<Track>
}

impl Player {
    pub fn new(
        d: Device, 
        track_rx: Receiver<Track>, 
        track_p_rx: Receiver<bool>, 
        track_i_x: Sender<bool>,
        track_atp_rx: Receiver<Track>
        ) -> Player {
        Player{
            handler: Sink::new(&d),
            device: d,
            track_rx,
            track_p_rx,
            track_i_x,
            track_atp_rx
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

