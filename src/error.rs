use core::fmt;

#[derive(Debug, Clone)]
pub struct MumbleLinkHandlerError {
    pub message: &'static str,
    pub os_error: bool,
}

impl fmt::Display for MumbleLinkHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}