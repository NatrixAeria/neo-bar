use super::config::{BarBuilder, DockDirection};
use super::error::RunnerError;
use super::event;

pub trait WmScreen {
    fn dimensions(&self) -> (u32, u32);
    fn physical_dimensions(&self) -> Option<(f32, f32)>;
}

pub trait WmAdapter<B: Bar>: Sized {
    type Error: std::error::Error;
    type Surface;
    type Screen<'s>: WmScreen;
    type AdapterBar<'a>: WmAdapterBar<'a, B, Self>;
    fn new(cfg: &BarBuilder) -> Result<Self, Self::Error>;
    fn get_screen_count(&self) -> usize;
    fn get_screen(&'_ self, n: usize) -> Option<Self::Screen<'_>>;
}

pub trait WmAdapterBar<'a, B: Bar, Wm: WmAdapter<B>>: Sized {
    fn new(
        bar: &B,
        wm: &'a Wm,
        cfg: &BarBuilder,
        screen: Wm::Screen<'a>,
    ) -> Result<Self, Wm::Error>;
    fn set_docking(&mut self, dir: DockDirection) -> Result<(), Wm::Error>;
    fn set_margin(&mut self, left: i32, right: i32) -> Result<(), Wm::Error>;
    fn blit(&mut self, surface: &Wm::Surface, x: i32, y: i32) -> Result<(), Wm::Error>;
    fn pop_event(&mut self) -> core::task::Poll<Result<event::Event, Wm::Error>>;
}

fn create_adapter_bar<'a, B: Bar, Wm: WmAdapter<B>>(
    bar: &B,
    wm: &'a Wm,
    builder: &BarBuilder,
    screen: Wm::Screen<'a>,
) -> Result<Wm::AdapterBar<'a>, Wm::Error> {
    Wm::AdapterBar::<'a>::new(bar, wm, builder, screen)
}

pub fn run<B: Bar, Wm: WmAdapter<B>>() -> Result<(), RunnerError<Wm::Error>> {
    let bar = B::new();
    let builder = bar.get_bar_builder();
    let wm = Wm::new(&builder)?;
    let mut bars = Vec::with_capacity(wm.get_screen_count());
    for i in 0..bars.capacity() {
        let screen = wm
            .get_screen(i)
            .ok_or_else(|| RunnerError::Custom(format!("failed to query screen {}", i)))?;
        bars.push(create_adapter_bar(&bar, &wm, &builder, screen)?);
    }
    loop {}
    Ok(())
}

pub trait Bar: Sized + 'static {
    fn new() -> Self;
    fn select_screens<S: WmScreen>(&self, screens: &[S]) -> Vec<usize> {
        (0..screens.len()).collect()
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

pub fn run_xcb<B: Bar>(
) -> Result<(), RunnerError<<crate::xcb::XcbAdapter<B> as WmAdapter<B>>::Error>> {
    run::<B, crate::xcb::XcbAdapter<B>>()
}
