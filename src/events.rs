use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

pub const TICK_RATE: Duration = Duration::from_millis(100);

pub enum Event {
    Tick,
    Key(KeyEvent),
}

pub struct Events {
    rx: Receiver<Event>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let has_event = event::poll(TICK_RATE).unwrap_or(false);
            if has_event {
                if let Ok(CrosstermEvent::Key(key)) = event::read() {
                if tx.send(Event::Key(key)).is_err() {
                    break;
                }
            }
            } else if tx.send(Event::Tick).is_err() {
                break;
            }
        });
        Self { rx }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}