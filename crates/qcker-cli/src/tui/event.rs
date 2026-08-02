use crossterm::event::{self, Event, KeyEvent, MouseEvent};

use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
    Mouse(MouseEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&self) -> anyhow::Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                Event::Key(key) => return Ok(AppEvent::Input(key)),
                Event::Mouse(mouse) => return Ok(AppEvent::Mouse(mouse)),
                _ => {}
            }
        }
        Ok(AppEvent::Tick)
    }
}
