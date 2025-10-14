extern crate core;

mod impls;

#[cfg(test)]
mod tests;

use std::io::{Read, Write};
use thiserror::Error;

pub use transmittable_derive::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("failed to decode UTF-8 string")]
    Utf8DecodeError(#[from] std::string::FromUtf8Error),
    #[error("invalid boolean (expected 0 or 1, got {0})")]
    InvalidBoolean(u8),
    #[error("invalid enum variant")]
    InvalidEnumVariant,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::IOError(e1),         Error::IOError(e2))         => e1.kind() == e2.kind() && e1.to_string() == e2.to_string(),
            (Error::Utf8DecodeError(e1), Error::Utf8DecodeError(e2)) => e1 == e2,
            (Error::InvalidBoolean(b1),  Error::InvalidBoolean(b2))  => b1 == b2,
            (Error::InvalidEnumVariant,  Error::InvalidEnumVariant)  => true,
            _ => false,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Transmittable {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> where Self: Sized;
}