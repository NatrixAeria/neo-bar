#[derive(Debug)]
pub enum Error {
    XcbError(xcb::GenericError),
    XcbConError(xcb::ConnError),
    Custom(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::XcbError(e) => write!(f, "xcb error: {}", e),
            Self::XcbConError(e) => write!(f, "xcb connection error: {}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

macro_rules! impl_err {
    ($a:ident, $b:ident) => {
        impl From<xcb::$a> for Error {
            fn from(e: xcb::$a) -> Self {
                Self::$b(e)
            }
        }
    };
}

impl_err!(ConnError, XcbConError);
impl_err!(GenericError, XcbError);
