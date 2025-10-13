use std::io::{Read, Write};
use crate::{Transmittable, Result};

macro_rules! impl_byte {
    ($($ty:ty),*) => {$(
        impl Transmittable for $ty {
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                writer.write_all(&[*self as u8])?;
                Ok(())
            }

            fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                Ok(buf[0] as Self)
            }
        }
    )*};
}

// This should be an optimization since we don't have to call `to_be_bytes` or `from_be_bytes`
// which is swapping bytes even though it contains a single byte.
impl_byte!(u8, i8);

// We're using big endian here since most network protocols use it.
// TODO: Impl the byteorder crate
macro_rules! impl_integer {
    ($($ty:ty),*) => {$(
        impl Transmittable for $ty {
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                writer.write_all(self.to_be_bytes().as_slice())?;
                Ok(())
            }

            fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
                let mut buf = [0u8; size_of::<Self>()];
                reader.read_exact(&mut buf)?;
                Ok(Self::from_be_bytes(buf))
            }
        }
    )*};
}

impl_integer!(
    u16, u32, u64, u128, usize,
    i16, i32, i64, i128, isize,
    f32, f64
);

impl Transmittable for bool {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&[if *self { 1 } else { 0 }])?;
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;

        if buf[0] == 0 {
            Ok(false)
        } else if buf[0] == 1 {
            Ok(true)
        } else {
            Err(crate::Error::InvalidBoolean(buf[0]))
        }
    }
}

// TODO: Somehow implement `char` without having to clone it

// TODO: This is bad for performance in a case where the elements are small and there is a lot of them, for example when reading a byte array
impl<T: Transmittable, const N: usize> Transmittable for [T; N] {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        Transmittable::serialize(&N, writer)?;

        for item in self {
            Transmittable::serialize(item, writer)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        // TODO: Allocate a slice instead, possibly with MaybeUninit?
        let size: usize = Transmittable::deserialize(reader)?;
        let mut buf: Vec<T> = Vec::with_capacity(size);

        for _ in 0..size {
            let item = Transmittable::deserialize(reader)?;
            buf.push(item);
        }

        buf.try_into().map_err(|_| unreachable!("Failed to convert a Vec<T> (of size N) to a [T; N]"))
    }
}

impl<T: Transmittable> Transmittable for Vec<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        Transmittable::serialize(&self.len(), writer)?;

        for item in self {
            Transmittable::serialize(item, writer)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let size = Transmittable::deserialize(reader)?;
        let mut buffer = Vec::with_capacity(size);

        for _ in 0..size {
            let item = Transmittable::deserialize(reader)?;
            buffer.push(item);
        }

        Ok(buffer)
    }
}

impl Transmittable for String {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        // TODO: Use slices instead, without having to create a vector
        Transmittable::serialize(&self.as_bytes().to_vec(), writer)?;
        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let buf = Transmittable::deserialize(reader)?;
        Ok(String::from_utf8(buf)?)
    }
}

impl<T: Transmittable> Transmittable for Option<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        Transmittable::serialize(&self.is_some(), writer)?;

        if let Some(item) = self {
            Transmittable::serialize(item, writer)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(reader: &mut R) -> Result<Self> {
        let is_present: bool = Transmittable::deserialize(reader)?;
        if is_present {
            Ok(Some(Transmittable::deserialize(reader)?))
        } else {
            Ok(None)
        }
    }
}