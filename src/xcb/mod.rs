mod error;
use core::convert::TryInto;
use core::task::Poll;
pub use error::Error;

use crate::bar::{Bar, WmAdapter, WmAdapterBar, WmScreen};
use crate::config::{BarBuilder, DockDirection};
use crate::event;

pub struct XcbAdapter<B: Bar> {
    con: xcb::Connection,
    _b: core::marker::PhantomData<B>,
}

pub struct XcbAdapterBar<'a, B: Bar> {
    dis: &'a XcbAdapter<B>,
    win: xcb::xproto::Window,
    _b: core::marker::PhantomData<&'a B>,
}

pub struct XcbScreen<'a> {
    handle: xcb::Screen<'a>,
}

pub enum LazyAtom<'s, 'a> {
    None(&'s str, xcb::xproto::InternAtomCookie<'a>),
    Ready(xcb::Atom),
}

impl<'s, 'a> LazyAtom<'s, 'a> {
    fn into_atom(self) -> Result<xcb::Atom, Error> {
        match self {
            Self::None(name, cookie) => match cookie.get_reply()?.atom() {
                xcb::ATOM_NONE => Err(Error::Custom(format!(
                    "missing intern X11 atom \"{}\"",
                    name
                ))),
                atom => Ok(atom),
            },
            Self::Ready(atom) => Ok(atom),
        }
    }

    pub fn get(&mut self) -> Result<xcb::Atom, Error> {
        let atom = core::mem::replace(self, Self::Ready(xcb::ATOM_NONE)).into_atom()?;
        *self = Self::Ready(atom);
        Ok(atom)
    }
}

impl<'a> XcbScreen<'a> {
    fn get_depth(&self, depth: u32, class: u8) -> Option<(xcb::Depth, xcb::Visualtype)> {
        self.handle
            .allowed_depths()
            .filter(|d| Into::<u32>::into(d.depth()) == depth)
            .find_map(|d| d.visuals().find(|v| v.class() == class).map(|v| (d, v)))
    }
}

impl<'a> WmScreen for XcbScreen<'a> {
    fn dimensions(&self) -> (u32, u32) {
        (
            self.handle.width_in_pixels().into(),
            self.handle.height_in_pixels().into(),
        )
    }
    fn physical_dimensions(&self) -> Option<(f32, f32)> {
        Some((
            self.handle.width_in_millimeters().into(),
            self.handle.height_in_millimeters().into(),
        ))
    }
}

impl<B: Bar> XcbAdapter<B> {
    fn get_intern_atom<'s, 'a>(&'a self, name: &'s str) -> LazyAtom<'s, 'a> {
        LazyAtom::None(name, xcb::intern_atom(&self.con, true, name))
    }
}

impl<B: Bar> WmAdapter<B> for XcbAdapter<B> {
    type Error = Error;
    type Surface = ();
    type Screen<'s> = XcbScreen<'s>;
    type AdapterBar<'a> = XcbAdapterBar<'a, B>;

    fn new(_cfg: &BarBuilder) -> Result<Self, Self::Error> {
        let (con, _screen_count) = xcb::Connection::connect(None).map_err(Error::XcbConError)?;
        Ok(Self {
            con,
            _b: core::marker::PhantomData,
        })
    }

    fn get_screen_count(&self) -> usize {
        self.con.get_setup().roots_len().into()
    }

    fn get_screen(&self, n: usize) -> Option<XcbScreen> {
        self.con
            .get_setup()
            .roots()
            .nth(n)
            .map(|s| XcbScreen { handle: s })
    }
}

type XSurface<B> = <XcbAdapter<B> as WmAdapter<B>>::Surface;

impl<'a, B: Bar> XcbAdapterBar<'a, B> {}

macro_rules! atoms32 {
    (assign($wm:expr, $win:expr) ($a:ident, $b:expr)) => {
        xcb::change_property(
            &$wm.con,
            xcb::PROP_MODE_REPLACE as u8,
            $win,
            $a.get()?,
            xcb::ATOM as u32,
            // Specifies whether the data should be viewed as a
            // list of 8-bit, 16-bit, or 32-bit quantities
            32,
            &$b,
        )
    };
    (assign($wm:expr, $win:expr) ($a:ident, $b:expr), $(($a2:ident, $b2:expr)),*) => {{
        let handle = atoms32!(assign($wm, $win) ($a, $b));
        atoms32!(assign($wm, $win) $(($a2, $b2)),*);
        handle.request_check()?;
    }};
    (_def($wm:expr) $a:ident) => { $wm.get_intern_atom(core::stringify!($a)); };
    (_def($wm:expr) [$($a:ident),*]) => { [$(atoms32!(_def($wm) $a)),*] };
    (def($wm:expr) [$(($a:ident, $b:ident)),*]) => { let [$(mut $a),*] = atoms32!(_def($wm) [$($b),*]); };
}

impl<'a, B: Bar> WmAdapterBar<'a, B, XcbAdapter<B>> for XcbAdapterBar<'a, B> {
    fn new(
        bar: &B,
        wm: &'a XcbAdapter<B>,
        cfg: &BarBuilder,
        screen: XcbScreen<'a>,
    ) -> Result<Self, Error> {
        let black_pixel = screen.handle.black_pixel();
        let geometry = xcb::xproto::get_geometry(&wm.con, screen.handle.root());
        let (visual, mut cw_values, depth_val) = if let (true, Some((depth, vis))) = (
            cfg.get_transparency(),
            screen.get_depth(32, xcb::VISUAL_CLASS_TRUE_COLOR as u8),
        ) {
            let colormap = wm.con.generate_id();
            xcb::create_colormap(
                &wm.con,
                xcb::COLORMAP_ALLOC_NONE as u8, // Colormap entries to be allocated (AllocNone or AllocAll)
                colormap,                       // Id of the color map
                screen.handle.root(), // Window on whose screen the colormap will be created
                vis.visual_id(),      // Id of the visual supported by the screen
            )
            .request_check()?;
            (
                vis.visual_id(), // visual
                vec![
                    (xcb::CW_COLORMAP, colormap),
                    (xcb::CW_BORDER_PIXEL, black_pixel),
                ], // cw_values
                depth.depth(),   // depth_val
            )
        } else {
            (
                screen.handle.root_visual(),
                Vec::with_capacity(3),
                xcb::COPY_FROM_PARENT as u8,
            )
        };
        let win = wm.con.generate_id();
        let geometry = geometry.get_reply()?;
        let (sw, sh) = (geometry.width(), geometry.height());
        let (x, y) = (geometry.x(), geometry.y());
        let width = (*cfg.get_width())
            .try_into()
            .map_err(|e| Error::Custom(format!("invalid bar width ({})", e)))?;
        let (w, h, grav) = match cfg.get_docking() {
            DockDirection::Top => (sw, width, xcb::GRAVITY_NORTH),
            DockDirection::Bottom => (sw, width, xcb::GRAVITY_SOUTH),
            DockDirection::Left => (width, sh, xcb::GRAVITY_WEST),
            DockDirection::Right => (width, sh, xcb::GRAVITY_EAST),
        };
        cw_values.push((xcb::CW_WIN_GRAVITY, grav));
        cw_values.push((xcb::CW_BACK_PIXEL, black_pixel));
        let events = bar.get_event_types();
        let f = |a, b| if events & a == 0 { 0 } else { b };
        cw_values.push((
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_EXPOSURE
                | f(
                    crate::event::CLICK,
                    xcb::EVENT_MASK_BUTTON_PRESS | xcb::EVENT_MASK_BUTTON_RELEASE,
                )
                | f(crate::event::MOUSE_MOVE, xcb::EVENT_MASK_BUTTON_MOTION),
        ));
        xcb::create_window(
            &wm.con,
            depth_val,
            win,
            screen.handle.root(),
            0,
            0,
            w,
            h,
            0,
            xcb::xproto::WINDOW_CLASS_INPUT_OUTPUT as u16,
            visual,
            &cw_values,
        )
        .request_check()?;

        let handle = xcb::map_window(&wm.con, win);

        atoms32! { def(wm) [
            (window_type, _NET_WM_WINDOW_TYPE),
            (dock, _NET_WM_WINDOW_TYPE_DOCK),
            (state, _NET_WM_STATE),
            (skip_taskbar, _NET_WM_STATE_SKIP_TASKBAR),
            (below, _NET_WM_STATE_BELOW),
            (strut, _NET_WM_STRUT),
            (strut_partial, _NET_WM_STRUT_PARTIAL)
        ]};
        let (left, right): (i16, i16) = (*cfg.get_margin_left())
            .try_into()
            .and_then(|a| (*cfg.get_margin_right()).try_into().map(|b| (a, b)))
            .map_err(|e| Error::Custom(format!("invalid bar outer margin ({})", e)))?;
        #[allow(clippy::deprecated_cfg_attr)]
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let strut_args: [i32; 12] = match cfg.get_docking() {
            DockDirection::Bottom => [0, 0, 0, width.into(), 0, 0, 0, 0, 0, 0, (x + left).into(), (x + (sw as i16) - right).into()],
            DockDirection::Top => [0, 0, width.into(), 0, 0, 0, 0, 0, (x + left).into(), (x + (sw as i16) - right).into(), 0, 0],
            DockDirection::Right => [0, width.into(), 0, 0, 0, 0, (y + left).into(), (y + (sh as i16) - right).into(), 0, 0, 0, 0],
            DockDirection::Left => [width.into(), 0, 0, 0, (y + left).into(), (y + (sh as i16) - right).into(), 0, 0, 0, 0, 0, 0],
        };
        atoms32! { assign(wm, win)
            (window_type, [dock.get()?]),
            (state, [skip_taskbar.get()?, below.get()?]),
            (strut, strut_args[..4]),
            (strut_partial, strut_args)
        };

        handle.request_check()?;
        wm.con.flush();

        Ok(Self {
            dis: wm,
            win,
            _b: core::marker::PhantomData,
        })
    }

    fn set_docking(&mut self, _dir: DockDirection) -> Result<(), Error> {
        unimplemented!("NIY")
    }

    fn set_margin(&mut self, _left: i32, _right: i32) -> Result<(), Error> {
        unimplemented!("NIY")
    }

    fn blit(&mut self, _surface: &XSurface<B>, _x: i32, _y: i32) -> Result<(), Error> {
        unimplemented!("NIY")
    }

    fn pop_event(&mut self) -> Poll<Result<event::Event, Error>> {
        unimplemented!("NIY")
    }
}
