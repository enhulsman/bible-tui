use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    tx: mpsc::Sender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();
        thread::spawn(move || loop {
            if event::poll(tick_rate).expect("failed to poll events") {
                match event::read().expect("failed to read event") {
                    CrosstermEvent::Key(e) => {
                        if event_tx.send(Event::Key(e)).is_err() {
                            return;
                        }
                    }
                    CrosstermEvent::Mouse(e) => {
                        if event_tx.send(Event::Mouse(e)).is_err() {
                            return;
                        }
                    }
                    CrosstermEvent::Resize(w, h) => {
                        if event_tx.send(Event::Resize(w, h)).is_err() {
                            return;
                        }
                    }
                    _ => {}
                }
            } else if event_tx.send(Event::Tick).is_err() {
                return;
            }
        });
        Self { rx, tx }
    }

    pub fn next(&self) -> color_eyre::Result<Event> {
        Ok(self.rx.recv()?)
    }
}
