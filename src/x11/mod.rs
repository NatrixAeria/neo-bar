mod error;
use core::convert::TryInto;
use core::task::Poll;
pub use error::Error;
use x11::protocol::xproto;
use x11rb as x11;
use xproto::ConnectionExt;

use crate::bar::{Bar, WmAdapter, WmAdapterBar, WmAdapterExt, WmAdapterGetBar, WmScreen};
use crate::config::{BarBuilder, DockDirection};
use crate::event;

pub type X11RustAdapter<B> = X11Adapter<B, x11rb::rust_connection::RustConnection>;
#[cfg(feature = "wm-x11-xcb")]
pub type X11XcbAdapter<B> = X11Adapter<B, x11rb::xcb_ffi::XCBConnection>;
pub type X11RustAdapterError<B> = <X11RustAdapter<B> as WmAdapter<B>>::Error;
#[cfg(feature = "wm-x11-xcb")]
pub type X11XcbAdapterError<B> = <X11XcbAdapter<B> as WmAdapter<B>>::Error;

const fn _assert_u32_u8_align() -> bool {
    core::mem::align_of::<u32>() == core::mem::align_of::<u8>() * 4
        && core::mem::size_of::<u32>() == core::mem::size_of::<u8>() * 4
}
const _ASSERT_U32_U8_ALIGN: u32 = [0][1 - (_assert_u32_u8_align() as usize)];

fn serialize_u32(arr: &[u32]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len() << 2) }
}

fn serialize_i32(arr: &[i32]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(arr.as_ptr() as *const u8, arr.len() << 2) }
}

x11::atom_manager! {
    Atoms: AtomCookies {
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_DOCK,
        _NET_WM_STATE,
        _NET_WM_STATE_SKIP_TASKBAR,
        _NET_WM_STATE_BELOW,
        _NET_WM_STATE_STICKY,
        _NET_WM_STRUT,
        _NET_WM_STRUT_PARTIAL,
        _NET_WM_DESKTOP,
        _NET_WM_ALLOWED_ACTIONS,
    }
}

pub trait X11Connection: x11rb::connection::Connection + Sized + 'static {
    fn connect(dpy_name: Option<&str>) -> Result<(Self, usize), Error>;
}

impl X11Connection for x11rb::rust_connection::RustConnection {
    fn connect(dpy_name: Option<&str>) -> Result<(Self, usize), Error> {
        Ok(x11rb::rust_connection::RustConnection::connect(dpy_name)?)
    }
}

#[cfg(feature = "wm-x11-xcb")]
impl X11Connection for x11rb::xcb_ffi::XCBConnection {
    fn connect(dpy_name: Option<&str>) -> Result<(Self, usize), Error> {
        let dpy_name = dpy_name
            .map(std::ffi::CString::new)
            .transpose()
            .map_err(|_| x11::errors::ConnectError::DisplayParsingError)?;
        let dpy_name = dpy_name.as_deref();
        Ok(x11::xcb_ffi::XCBConnection::connect(dpy_name)?)
    }
}

#[derive(Debug, Clone)]
pub struct X11Adapter<B: Bar, C: X11Connection> {
    con: C,
    atoms: Atoms,
    _b: core::marker::PhantomData<B>,
}

#[derive(Debug, Clone)]
pub struct X11AdapterBar<'a, B: Bar, C: X11Connection> {
    dis: &'a X11Adapter<B, C>,
    win: xproto::Window,
    left: i16,
    right: i16,
    pos: (i16, i16),
    screen_size: (u16, u16),
    size: (u16, u16),
    width: u16,
}

impl<'a, B: Bar, C: X11Connection> WmAdapterGetBar<'a, B> for X11Adapter<B, C> {
    type AdapterBar = X11AdapterBar<'a, B, C>;
}

impl<B: Bar, C: X11Connection> WmAdapterExt<B> for X11Adapter<B, C> {}

pub struct Surface;

impl WmScreen for xproto::Screen {
    fn dimensions(&self) -> (u32, u32) {
        (self.width_in_pixels.into(), self.height_in_pixels.into())
    }
    fn physical_dimensions(&self) -> Option<(f32, f32)> {
        Some((
            self.width_in_millimeters.into(),
            self.height_in_millimeters.into(),
        ))
    }
}

impl<B: Bar, C: X11Connection> X11Adapter<B, C> {
    fn map_event(&self, ev: x11::protocol::Event) -> Option<event::Event> {
        use x11::protocol::Event::*;
        Some(match ev {
            ButtonPress(ev) => event::Event::MouseDown(event::ClickEvent {
                x: ev.root_x.into(),
                y: ev.root_y.into(),
            }),
            ButtonRelease(ev) => event::Event::MouseUp(event::ClickEvent {
                x: ev.root_x.into(),
                y: ev.root_y.into(),
            }),
            _ => return (None, dbg!(ev)).0,
        })
    }
}

impl<B: Bar, C: X11Connection> WmAdapter<B> for X11Adapter<B, C> {
    type Error = Error;
    type Surface = Surface;
    type Screen = xproto::Screen;

    fn new(_cfg: &BarBuilder) -> Result<Self, Self::Error> {
        let (con, _screen_count) = C::connect(None)?;
        let atoms = Atoms::new(&con)?.reply()?;
        Ok(Self {
            con,
            atoms,
            _b: core::marker::PhantomData,
        })
    }

    fn get_screen_count(&self) -> usize {
        self.con.setup().roots_len().into()
    }

    fn get_screen(&self, n: usize) -> Option<&xproto::Screen> {
        self.con.setup().roots.get(n)
    }

    fn await_event(&self) -> Result<event::Event, Self::Error> {
        loop {
            return Ok(match self.map_event(self.con.wait_for_event()?) {
                Some(v) => v,
                _ => continue,
            });
        }
    }

    fn poll_event(&self) -> Result<Poll<event::Event>, Self::Error> {
        Ok(
            match self.con.poll_for_event()?.and_then(|v| self.map_event(v)) {
                Some(v) => Poll::Ready(v),
                None => Poll::Pending,
            },
        )
    }
}

fn filter_depth_visual_rgba(screen: &xproto::Screen) -> Option<(u8, &xproto::Visualtype)> {
    screen
        .allowed_depths
        .iter()
        .filter(|d| d.depth == 32)
        .find_map(|d| {
            d.visuals
                .iter()
                .find(|v| v.class == xproto::VisualClass::TrueColor && v.bits_per_rgb_value == 32)
                .map(|v| (d.depth, v))
        })
}

impl<'a, B: Bar, C: X11Connection> WmAdapterBar<'a, B, X11Adapter<B, C>>
    for X11AdapterBar<'a, B, C>
{
    fn new(
        bar: &B,
        wm: &'a X11Adapter<B, C>,
        cfg: &BarBuilder,
        screen: &xproto::Screen,
    ) -> Result<Self, Error> {
        let geometry = wm.con.get_geometry(screen.root)?;
        let (visual, cw_values, depth_val) = if let (true, Some((depth, vis))) =
            (cfg.get_transparency(), filter_depth_visual_rgba(&screen))
        {
            let colormap = wm.con.generate_id()?;
            wm.con
                .create_colormap(
                    xproto::ColormapAlloc::None,
                    colormap,
                    screen.root,
                    vis.visual_id,
                )?
                .check()?;
            (
                vis.visual_id,
                xproto::CreateWindowAux::new()
                    .colormap(colormap)
                    .border_pixel(screen.black_pixel),
                depth,
            )
        } else {
            (
                screen.root_visual,
                xproto::CreateWindowAux::new(),
                x11::COPY_DEPTH_FROM_PARENT as u8,
            )
        };
        let win = wm.con.generate_id()?;
        let geometry = geometry.reply()?;
        let (sw, sh) = (geometry.width, geometry.height);
        let (x, y) = (geometry.x, geometry.y);
        let width = (*cfg.get_width())
            .try_into()
            .map_err(|e| Error::Custom(format!("invalid bar width ({})", e)))?;
        let (w, h, grav) = match cfg.get_docking() {
            DockDirection::Top => (sw, width, xproto::Gravity::North),
            DockDirection::Bottom => (sw, width, xproto::Gravity::South),
            DockDirection::Left => (width, sh, xproto::Gravity::West),
            DockDirection::Right => (width, sh, xproto::Gravity::East),
        };
        let events = bar.get_event_types();
        let f = |a, b| {
            if events & a == 0 {
                xproto::EventMask::NoEvent as u32
            } else {
                b
            }
        };
        let cw_values = cw_values
            .bit_gravity(grav)
            .win_gravity(grav)
            .background_pixel(screen.black_pixel)
            .event_mask(
                xproto::EventMask::Exposure
                    | f(
                        crate::event::CLICK,
                        xproto::EventMask::ButtonPress | xproto::EventMask::ButtonRelease,
                    )
                    | f(
                        crate::event::MOUSE_MOVE,
                        xproto::EventMask::ButtonMotion as u32,
                    ),
            );
        wm.con
            .create_window(
                depth_val,
                win,
                screen.root,
                x,
                y,
                w,
                h,
                0,
                xproto::WindowClass::InputOutput,
                visual,
                &cw_values,
            )?
            .check()?;

        let cookie1 = wm.con.map_window(win)?;
        let cookie2 = wm.con.configure_window(
            win,
            &xproto::ConfigureWindowAux::new()
                .x(Some(x.into()))
                .y(Some(y.into()))
                .width(Some(w.into()))
                .height(Some(h.into())),
        )?;

        let (left, right): (i16, i16) = (*cfg.get_margin_left())
            .try_into()
            .and_then(|a| (*cfg.get_margin_right()).try_into().map(|b| (a, b)))
            .map_err(|e| Error::Custom(format!("invalid bar outer margin ({})", e)))?;

        let slf = Self {
            left,
            right,
            pos: (x, y),
            size: (w, h),
            screen_size: (sw, sh),
            dis: wm,
            win,
            width,
        };

        let atoms = &slf.dis.atoms;
        let docking_cookies = slf.set_docking_cookie(*cfg.get_docking())?;
        let cookies = [
            slf.change_property_atoms(
                atoms._NET_WM_WINDOW_TYPE,
                &[atoms._NET_WM_WINDOW_TYPE_DOCK],
            )?,
            slf.change_property_atoms(
                atoms._NET_WM_STATE,
                &[
                    atoms._NET_WM_STATE_STICKY,
                    atoms._NET_WM_STATE_SKIP_TASKBAR,
                    atoms._NET_WM_STATE_BELOW,
                ],
            )?,
            slf.change_property_u32(atoms._NET_WM_DESKTOP, &[0xFFFFFFFF])?,
            slf.change_property_atoms(atoms._NET_WM_ALLOWED_ACTIONS, &[])?,
        ];
        slf.await_void_cookies(docking_cookies)?;
        slf.await_void_cookies(cookies)?;

        cookie1.check()?;
        cookie2.check()?;
        wm.con.flush()?;

        Ok(slf)
    }

    fn set_docking(&mut self, dir: DockDirection) -> Result<(), Error> {
        self.await_void_cookies(self.set_docking_cookie(dir)?)
    }

    fn set_margin(&mut self, _left: i32, _right: i32) -> Result<(), Error> {
        unimplemented!("NIY")
    }

    fn blit(&mut self, _surface: &Surface, _x: i32, _y: i32) -> Result<(), Error> {
        unimplemented!("NIY")
    }
}

impl<'a, B: Bar, C: X11Connection> X11AdapterBar<'a, B, C> {
    fn await_void_cookies<const N: usize>(
        &self,
        cookies: [x11::cookie::VoidCookie<'a, C>; N],
    ) -> Result<(), Error> {
        Ok(for cookie in core::array::IntoIter::new(cookies) {
            cookie.check()?;
        })
    }
    fn change_property_u32(
        &self,
        key: xproto::Atom,
        values: &[u32],
    ) -> Result<x11::cookie::VoidCookie<'a, C>, Error> {
        Ok(self.dis.con.change_property(
            xproto::PropMode::Replace,
            self.win,
            key,
            xproto::AtomEnum::CARDINAL,
            32,
            values.len() as u32,
            serialize_u32(values),
        )?)
    }
    fn change_property_i32(
        &self,
        key: xproto::Atom,
        values: &[i32],
    ) -> Result<x11::cookie::VoidCookie<'a, C>, Error> {
        Ok(self.dis.con.change_property(
            xproto::PropMode::Replace,
            self.win,
            key,
            xproto::AtomEnum::CARDINAL,
            32,
            values.len() as u32,
            serialize_i32(values),
        )?)
    }
    fn change_property_atoms(
        &self,
        key: xproto::Atom,
        values: &[xproto::Atom],
    ) -> Result<x11::cookie::VoidCookie<'a, C>, Error> {
        Ok(self.dis.con.change_property(
            xproto::PropMode::Replace,
            self.win,
            key,
            xproto::AtomEnum::ATOM,
            32,
            values.len() as u32,
            serialize_u32(values),
        )?)
    }
    fn set_docking_cookie(
        &self,
        dir: DockDirection,
    ) -> Result<[x11::cookie::VoidCookie<'a, C>; 2], Error> {
        let ((x, y), (sw, sh), width, left, right) = (
            self.pos,
            self.screen_size,
            self.width,
            self.left,
            self.right,
        );
        #[allow(clippy::deprecated_cfg_attr)]
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let strut_args: [i32; 12] = match dir {
            DockDirection::Bottom => [0, 0, 0, width.into(), 0, 0, 0, 0, 0, 0, (x + left).into(), (x + (sw as i16) - right).into()],
            DockDirection::Top => [0, 0, width.into(), 0, 0, 0, 0, 0, (x + left).into(), (x + (sw as i16) - right).into(), 0, 0],
            DockDirection::Right => [0, width.into(), 0, 0, 0, 0, (y + left).into(), (y + (sh as i16) - right).into(), 0, 0, 0, 0],
            DockDirection::Left => [width.into(), 0, 0, 0, (y + left).into(), (y + (sh as i16) - right).into(), 0, 0, 0, 0, 0, 0],
        };
        Ok([
            self.change_property_i32(self.dis.atoms._NET_WM_STRUT, &strut_args[..4])?,
            self.change_property_i32(self.dis.atoms._NET_WM_STRUT_PARTIAL, &strut_args)?,
        ])
    }
}
