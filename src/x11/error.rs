#[derive(Debug)]
pub enum Error {
    X11Error(x11rb::errors::ReplyOrIdError),
    X11ConError(x11rb::errors::ConnectError),
    Custom(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::X11Error(e) => write!(f, "xcb error: {}", e),
            Self::X11ConError(e) => write!(f, "xcb connect error: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

macro_rules! impl_err {
    ($a:ident, $b:ident, $f:expr) => {
        impl From<x11rb::errors::$a> for Error {
            fn from(e: x11rb::errors::$a) -> Self {
                Self::$b($f(e))
            }
        }
    };
    ($a:ident, $b:ident) => { impl_err!($a, $b, (|e| e)); };
}

impl_err!(ConnectError, X11ConError);
impl_err!(ReplyError, X11Error, Into::into);
impl_err!(ConnectionError, X11Error, Into::into);
impl_err!(ReplyOrIdError, X11Error);
