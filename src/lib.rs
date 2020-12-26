#![feature(array_value_iter)]
#![feature(min_const_generics)]

pub mod bar;
pub mod config;
pub mod error;
pub mod event;
pub mod x11;

#[cfg(not(feature = "wm-x11-xcb"))]
pub use bar::run_x11 as run;
#[cfg(feature = "wm-x11-xcb")]
pub use bar::run_x11_xcb as run;
