extern crate core;

mod impls;

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

pub type Result<T> = std::result::Result<T, Error>;

pub trait Transmittable {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()>;
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> where Self: Sized;
}

#[derive(Transmittable)]
#[repr(u8)]
pub enum MyEnum {
    Foo,
    Bar = 6,
    FooBar,
}