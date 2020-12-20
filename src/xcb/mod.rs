mod error;
use core::task::Poll;
pub use error::Error;

use crate::bar::{Bar, WmAdapter, WmAdapterBar};
use crate::config::{BarBuilder, DockDirection};
use crate::event;

pub struct XcbAdapter<B: Bar> {
    con: xcb::Connection,
    _b: core::marker::PhantomData<B>,
}

pub struct XcbAdapterBar<'a, B: Bar> {
    _b: core::marker::PhantomData<&'a B>,
}

impl<B: Bar> XcbAdapter<B> {
    pub fn query_screen_count(&self) -> usize {
        self.con.get_setup().roots_len().into()
    }
}

impl<B: Bar> WmAdapter<B> for XcbAdapter<B> {
    type Error = Error;
    type Surface = ();
    type AdapterBar<'a> = XcbAdapterBar<'a, B>;

    fn new(_cfg: &BarBuilder) -> Result<Self, Self::Error> {
        println!("yeah");
        let (con, _screen_count) = xcb::Connection::connect(None).map_err(Error::XcbConError)?;
        Ok(Self {
            con,
            _b: core::marker::PhantomData,
        })
    }
}

type XError<B> = <XcbAdapter<B> as WmAdapter<B>>::Error;
type XSurface<B> = <XcbAdapter<B> as WmAdapter<B>>::Surface;

impl<'a, B: Bar> WmAdapterBar<'a, B, XcbAdapter<B>> for XcbAdapterBar<'a, B> {
    fn new(wm: &'a XcbAdapter<B>, cfg: &BarBuilder) -> Result<Self, XError<B>> {
        unimplemented!("NIY")
    }

    fn set_docking(&mut self, _dir: DockDirection) -> Result<(), XError<B>> {
        unimplemented!("NIY")
    }

    fn set_margin(&mut self, _left: i32, _right: i32) -> Result<(), XError<B>> {
        unimplemented!("NIY")
    }

    fn blit(&mut self, _surface: &XSurface<B>, _x: i32, _y: i32) -> Result<(), XError<B>> {
        unimplemented!("NIY")
    }

    fn pop_event(&mut self) -> Poll<Result<event::Event, XError<B>>> {
        unimplemented!("NIY")
    }
}
