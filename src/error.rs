#[derive(Debug)]
pub enum RunnerError<E: std::error::Error> {
    WmError(E),
    Custom(String),
}

impl<E: std::error::Error> std::fmt::Display for RunnerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::WmError(e) => write!(f, "{}", e),
            Self::Custom(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error> From<E> for RunnerError<E> {
    fn from(e: E) -> Self {
        Self::WmError(e)
    }
}
