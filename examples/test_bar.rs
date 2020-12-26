use neo_bar::bar::{Bar, WmAdapterExt, WmAdapterGetBar};
use neo_bar::config::{BarBuilder, DockDirection, ZIndex};
use neo_bar::event;

pub struct TestBar;

impl Bar for TestBar {
    fn new() -> Self {
        Self
    }

    fn get_bar_builder(&self) -> BarBuilder {
        BarBuilder::default()
            .title("test")
            .docking(DockDirection::Bottom)
            .margin(0, 0)
            .z_index(ZIndex::AboveEverything)
            .transparency(true)
            .width(35)
    }

    fn get_event_types(&self) -> event::EventTypes {
        event::CLICK | event::QUIT
    }

    fn on_click<'a, Wm: WmAdapterExt<Self>>(
        &mut self,
        _bar: &mut <Wm as WmAdapterGetBar<'a, Self>>::AdapterBar,
        event: event::ClickEvent,
    ) {
        println!("click {:?}", event);
    }

    fn on_quit(&mut self) {
        println!("quit!");
    }
}

fn main() {
    if let Err(e) = neo_bar::run::<TestBar>() {
        println!("\x1b[1;31mfatal error: {}", e);
    }
}
