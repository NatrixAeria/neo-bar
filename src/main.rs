#![feature(array_value_iter)]

pub mod bar;
pub mod config;
pub mod error;
pub mod event;
pub mod x11;

pub struct TestBar;

impl bar::Bar for TestBar {
    fn new() -> Self {
        Self
    }

    fn get_bar_builder(&self) -> config::BarBuilder {
        config::BarBuilder::default()
            .title("test")
            .docking(config::DockDirection::Bottom)
            .margin(0, 0)
            .z_index(config::ZIndex::AboveEverything)
            .transparency(true)
            .width(35)
    }

    fn get_event_types(&self) -> event::EventTypes {
        event::CLICK | event::QUIT
    }

    fn on_click<'a, Wm: bar::WmAdapterExt<Self>>(
        &mut self,
        _bar: &mut <Wm as bar::WmAdapterGetBar<'a, Self>>::AdapterBar,
        event: event::ClickEvent,
    ) {
        println!("click {:?}", event);
    }

    fn on_quit(&mut self) {
        println!("quit!");
    }
}

fn main() {
    if let Err(e) = bar::run_x11::<TestBar>() {
        println!("\x1b[1;31mfatal error: {}", e);
    }
}
