pub mod bar;
pub mod config;
pub mod event;

pub struct TestBar;

impl bar::Bar for TestBar {
    fn new() -> Self {
        Self
    }

    fn bar_builder(&mut self) -> config::BarBuilder {
        config::BarBuilder::default()
            .title("test")
            .docking(config::DockDirection::Left)
            .margin_left(50)
            .margin_right(50)
            .z_index(config::ZIndex::AboveEverything)
    }

    fn get_event_types(&mut self) -> event::EventTypes {
        event::CLICK | event::QUIT
    }

    fn on_click<Wm: bar::WmAdapter<Self>>(&mut self, _bar: &mut Wm, event: event::ClickEvent) {
        println!("click {:?}", event);
    }

    fn on_quit(&mut self) {
        println!("quit!");
    }
}

fn main() {}
