use super::config::{BarBuilder, DockDirection};
use super::event;

#[async_trait::async_trait]
pub trait WmAdapter<B: Bar>: Sized {
    type Error: std::error::Error;
    type Surface;
    fn new(cfg: BarBuilder) -> Result<Self, Self::Error>;
    fn set_docking(&mut self, dir: DockDirection) -> Result<(), Self::Error>;
    fn set_margin(&mut self, left: i32, right: i32) -> Result<(), Self::Error>;
    fn blit(&mut self, surface: &Self::Surface, x: i32, y: i32) -> Result<(), Self::Error>;
    async fn pop_event(&mut self) -> Result<event::Event, Self::Error>;
}

pub struct BarHandler<B: Bar, Wm: WmAdapter<B>> {
    bar: B,
    bars: Vec<Wm>,
    _bar: core::marker::PhantomData<B>,
}

impl<B: Bar, Wm: WmAdapter<B>> BarHandler<B, Wm> {
    pub fn run() -> Result<(), Wm::Error> {
        Ok(())
    }
}

pub trait Bar: Sized {
    fn new() -> Self;
    fn bar_builder(&mut self) -> BarBuilder {
        BarBuilder::default()
    }
    fn get_event_types(&mut self) -> event::EventTypes {
        0
    }
    fn on_window_open<Wm: WmAdapter<Self>>(&mut self, _bar: &mut Wm) {}
    fn on_click<Wm: WmAdapter<Self>>(&mut self, _bar: &mut Wm, _event: event::ClickEvent) {}
    fn on_quit(&mut self) {}
}
