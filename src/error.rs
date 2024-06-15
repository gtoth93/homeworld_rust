//! Error handling module

use std::fmt::{Display, Formatter};

use derive_more::From;

use crate::bigfile;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    ConversionError,
    #[from]
    BigFile(bigfile::Error),
    #[from]
    Io(std::io::Error),
    #[from]
    PodCast(bytemuck::PodCastError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for Error {}
