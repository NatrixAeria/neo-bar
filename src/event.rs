#[derive(Debug, Clone)]
pub struct ClickEvent {}

#[derive(Debug, Clone)]
pub enum Event {
    Click(ClickEvent),
}

pub type EventTypes = u32;
pub const CLICK: u32 = 1;
pub const QUIT: u32 = 2;
