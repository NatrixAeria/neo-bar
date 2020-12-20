#[derive(Debug)]
pub enum Error {
    XcbConError(xcb::ConnError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::XcbConError(e) => write!(f, "xcb connection error: {}", e),
        }
    }
}

impl std::error::Error for Error {}