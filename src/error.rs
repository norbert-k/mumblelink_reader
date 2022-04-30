use thiserror::Error;

#[derive(Error, Debug)]
pub enum MumbleLinkHandlerError {
    #[error("Read error (pointer: 0x0)")]
    UnableToRead,
    #[error(transparent)]
    OSError(#[from] std::io::Error),
}
