#[derive(Debug, Clone)]
pub struct ClickEvent {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseDown(ClickEvent),
    MouseUp(ClickEvent),
}

pub type EventTypes = u32;
pub const QUIT: u32 = 1;
pub const CLICK: u32 = 2;
pub const MOUSE_MOVE: u32 = 4;
