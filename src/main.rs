#![allow(incomplete_features)]
#![feature(generic_associated_types)]

pub mod bar;
pub mod config;
pub mod event;
pub mod xcb;

pub struct TestBar;

impl bar::Bar for TestBar {
    fn new() -> Self {
        Self
    }

    fn get_screens(&self) -> bar::Screens {
        bar::Screens::AllScreens
    }

    fn get_bar_builder(&self) -> config::BarBuilder {
        config::BarBuilder::default()
            .title("test")
            .docking(config::DockDirection::Left)
            .margin_left(50)
            .margin_right(50)
            .z_index(config::ZIndex::AboveEverything)
    }

    fn get_event_types(&self) -> event::EventTypes {
        event::CLICK | event::QUIT
    }

    fn on_click<'a, Wm: bar::WmAdapter<Self>, WmBar: bar::WmAdapterBar<'a, Self, Wm>>(
        &mut self,
        _bar: &'a mut WmBar,
        event: event::ClickEvent,
    ) {
        println!("click {:?}", event);
    }

    fn on_quit(&mut self) {
        println!("quit!");
    }
}

fn main() {
    if let Err(e) = bar::run_xcb::<TestBar>() {
        println!("\x1b[1;31mfatal error: {}", e);
    }
}
