use super::config::{BarBuilder, DockDirection};
use super::event;

pub type Screen = usize;

pub enum Screens {
    AllScreens,
    MainScreen,
    Screens(Vec<Screen>),
}

pub trait WmAdapter<B: Bar>: Sized {
    type Error: std::error::Error;
    type Surface;
    type AdapterBar<'a>: WmAdapterBar<'a, B, Self>;
    fn new(cfg: &BarBuilder) -> Result<Self, Self::Error>;
}

pub trait WmAdapterBar<'a, B: Bar, Wm: WmAdapter<B>>: Sized {
    fn new(wm: &'a Wm, cfg: &BarBuilder) -> Result<Self, Wm::Error>;
    fn set_docking(&mut self, dir: DockDirection) -> Result<(), Wm::Error>;
    fn set_margin(&mut self, left: i32, right: i32) -> Result<(), Wm::Error>;
    fn blit(&mut self, surface: &Wm::Surface, x: i32, y: i32) -> Result<(), Wm::Error>;
    fn pop_event(&mut self) -> core::task::Poll<Result<event::Event, Wm::Error>>;
}

fn create_adapter_bar<'a, B: Bar, Wm: WmAdapter<B>>(
    wm: &'a Wm,
    builder: &BarBuilder,
) -> Result<Wm::AdapterBar<'a>, Wm::Error> {
    Wm::AdapterBar::<'a>::new(wm, builder)
}

pub fn run<B: Bar, Wm: WmAdapter<B>>() -> Result<(), Wm::Error> {
    let bar = B::new();
    let builder = bar.get_bar_builder();
    let wm = Wm::new(&builder)?;
    let _wm_bar = create_adapter_bar(&wm, &builder)?;
    Ok(())
}

pub trait Bar: Sized + 'static {
    fn new() -> Self;
    fn get_screens(&self) -> Screens {
        Screens::AllScreens
    }
    fn get_bar_builder(&self) -> BarBuilder {
        BarBuilder::default()
    }
    fn get_event_types(&self) -> event::EventTypes {
        0
    }
    fn on_window_open<Wm: WmAdapter<Self>>(&mut self, _bar: &mut Wm) {}
    fn on_click<'a, Wm: WmAdapter<Self>, WmBar: WmAdapterBar<'a, Self, Wm>>(
        &mut self,
        _bar: &'a mut WmBar,
        _event: event::ClickEvent,
    ) {
    }
    fn on_quit(&mut self) {}
}

pub fn run_xcb<B: Bar>() -> Result<(), <crate::xcb::XcbAdapter<B> as WmAdapter<B>>::Error> {
    run::<B, crate::xcb::XcbAdapter<B>>()
}
