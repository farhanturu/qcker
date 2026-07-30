use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
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
            if let Event::Key(key) = event::read()? {
                return Ok(AppEvent::Input(key));
            }
        }
        Ok(AppEvent::Tick)
    }
}
