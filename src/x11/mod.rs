mod error;
mod wm;

#[doc(inline)]
pub use wm::*;

pub type X11RustAdapter<B> = X11Adapter<B, x11rb::rust_connection::RustConnection>;
#[cfg(feature = "wm-x11-xcb")]
pub type X11XcbAdapter<B> = X11Adapter<B, x11rb::xcb_ffi::XCBConnection>;
pub type X11RustAdapterError<B> = <X11RustAdapter<B> as super::bar::WmAdapter<B>>::Error;
#[cfg(feature = "wm-x11-xcb")]
pub type X11XcbAdapterError<B> = <X11XcbAdapter<B> as super::bar::WmAdapter<B>>::Error;
