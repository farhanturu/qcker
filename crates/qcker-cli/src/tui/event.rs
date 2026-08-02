use crossterm::event::{self, Event, KeyEvent, MouseEvent, MouseEventKind};
use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
    Click(MouseEvent),
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
            if let Event::Mouse(mouse) = event::read()? {
                return Ok(AppEvent::Click(mouse));
            }
        }
        Ok(AppEvent::Tick)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickRegion {
    Tab(usize),
    ContainerRow(usize),
    ImageRow(usize),
    NetworkRow(usize),
    VolumeRow(usize),
    FileRow(usize),
    MarketplaceRow(usize),
    LogRow(usize),
    ActionButton(usize),
    DetailButton(usize),
    HelpButton,
    CloseButton,
}

impl ClickRegion {
    pub fn to_u16(&self) -> u16 {
        match self {
            ClickRegion::Tab(i) => *i as u16,
            ClickRegion::ContainerRow(i) => 100 + *i as u16,
            ClickRegion::ImageRow(i) => 200 + *i as u16,
            ClickRegion::NetworkRow(i) => 300 + *i as u16,
            ClickRegion::VolumeRow(i) => 400 + *i as u16,
            ClickRegion::FileRow(i) => 500 + *i as u16,
            ClickRegion::MarketplaceRow(i) => 600 + *i as u16,
            ClickRegion::LogRow(i) => 700 + *i as u16,
            ClickRegion::ActionButton(i) => 800 + *i as u16,
            ClickRegion::DetailButton(i) => 900 + *i as u16,
            ClickRegion::HelpButton => 950,
            ClickRegion::CloseButton => 951,
        }
    }
}
