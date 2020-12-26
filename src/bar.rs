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
    type Screen: WmScreen;
    fn new(cfg: &BarBuilder) -> Result<Self, Self::Error>;
    fn get_screen_count(&self) -> usize;
    fn get_screen(&self, n: usize) -> Option<&Self::Screen>;
    fn await_event(&self) -> Result<event::Event, Self::Error>;
    fn poll_event(&self) -> Result<core::task::Poll<event::Event>, Self::Error>;
}

pub trait WmAdapterBar<'a, B: Bar, Wm: WmAdapter<B>>: Sized {
    fn new(bar: &B, wm: &'a Wm, cfg: &BarBuilder, screen: &Wm::Screen) -> Result<Self, Wm::Error>;
    fn set_docking(&mut self, dir: DockDirection) -> Result<(), Wm::Error>;
    fn set_margin(&mut self, left: i32, right: i32) -> Result<(), Wm::Error>;
    fn blit(&mut self, surface: &Wm::Surface, x: i32, y: i32) -> Result<(), Wm::Error>;
}

pub trait WmAdapterGetBar<'a, B: Bar>: WmAdapter<B> {
    type AdapterBar: WmAdapterBar<'a, B, Self>;
}

pub trait WmAdapterExt<B: Bar>: WmAdapter<B> + for<'a> WmAdapterGetBar<'a, B> {}

pub fn run<B: Bar, Wm: WmAdapterExt<B>>() -> Result<(), RunnerError<Wm::Error>> {
    let mut bar = B::new();
    let builder = bar.get_bar_builder();
    let mut wm = Wm::new(&builder)?;
    let mut bars = Vec::with_capacity(wm.get_screen_count());
    for i in 0..bars.capacity() {
        let screen = wm
            .get_screen(i)
            .ok_or_else(|| RunnerError::Custom(format!("failed to query screen {}", i)))?;
        fn create_bar<'a, B: Bar, Wm: WmAdapterExt<B>>(
            bar: &B,
            wm: &'a Wm,
            builder: &crate::config::BarBuilder,
            screen: &Wm::Screen,
        ) -> Result<<Wm as WmAdapterGetBar<'a, B>>::AdapterBar, Wm::Error> {
            <Wm as WmAdapterGetBar<'a, B>>::AdapterBar::new(bar, wm, builder, screen)
        }
        bars.push(create_bar(&bar, &wm, &builder, screen)?);
    }
    loop {
        let ev = wm.await_event()?;
        match ev {
            event::Event::MouseUp(ev) | event::Event::MouseDown(ev) => {
                bar.on_click::<Wm>(bars.get_mut(0).unwrap(), ev);
            }
        }
    }
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
    fn on_bar_start<'a, Wm: WmAdapterExt<Self>>(
        &mut self,
        _bar: &mut <Wm as WmAdapterGetBar<'a, Self>>::AdapterBar,
    ) {
    }
    fn on_click<'a, Wm: WmAdapterExt<Self>>(
        &mut self,
        _bar: &mut <Wm as WmAdapterGetBar<'a, Self>>::AdapterBar,
        _event: event::ClickEvent,
    ) {
    }
    fn on_quit(&mut self) {}
}

pub fn run_x11<B: Bar>() -> Result<(), RunnerError<crate::x11::X11RustAdapterError<B>>> {
    run::<B, crate::x11::X11RustAdapter<B>>()
}

#[cfg(feature = "wm-x11-xcb")]
pub fn run_x11_xcb<B: Bar>(
) -> Result<(), RunnerError<RunnerError<crate::x11::X11XcbAdapterError<B>>>> {
    run::<B, crate::x11::X11XcbAdapter<B, x11rb::xcb_ffi::XCBConnection>>()
}
