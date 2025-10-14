use crate::Error;
use transmittable_macros::read_and_write;

read_and_write!(bool;
    (&[0u8],  Ok(false)),
    (&[1u8],  Ok(true)),
    (&[2u8],  Err(Error::InvalidBoolean(2))),
    (b"\xFF", Err(Error::InvalidBoolean(255))),
);


read_and_write!(u8;    (b"\x00", Ok(0)),                                                             (b"\xFF",                                                             Ok(u8::MAX)));
read_and_write!(u16;   (b"\x00\x00", Ok(0)),                                                         (b"\xFF\xFF",                                                         Ok(u16::MAX)));
read_and_write!(u32;   (b"\x00\x00\x00\x00", Ok(0)),                                                 (b"\xFF\xFF\xFF\xFF",                                                 Ok(u32::MAX)));
read_and_write!(u64;   (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0)),                                 (b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",                                 Ok(u64::MAX)));
read_and_write!(u128;  (b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0)), (b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF", Ok(u128::MAX)));
#[cfg(target_pointer_width = "32")]
read_and_write!(usize; (b"\x00\x00\x00\x00", Ok(0)),                                                 (b"\xFF\xFF\xFF\xFF",                                                 Ok(usize::MAX)));
#[cfg(target_pointer_width = "64")]
read_and_write!(usize; (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0)),                                 (b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF",                                 Ok(usize::MAX)));


read_and_write!(i8;    (b"\x00", Ok(0)),                                                             (b"\x7F",                                                             Ok(i8::MAX)));
read_and_write!(i16;   (b"\x00\x00", Ok(0)),                                                         (b"\x7F\xFF",                                                         Ok(i16::MAX)));
read_and_write!(i32;   (b"\x00\x00\x00\x00", Ok(0)),                                                 (b"\x7F\xFF\xFF\xFF",                                                 Ok(i32::MAX)));
read_and_write!(i64;   (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0)),                                 (b"\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF",                                 Ok(i64::MAX)));
read_and_write!(i128;  (b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0)), (b"\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF", Ok(i128::MAX)));
#[cfg(target_pointer_width = "32")]
read_and_write!(isize; (b"\x00\x00\x00\x00", Ok(0)),                                                 (b"\x7F\xFF\xFF\xFF",                                                 Ok(isize::MAX)));
#[cfg(target_pointer_width = "64")]
read_and_write!(isize; (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0)),                                 (b"\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF",                                 Ok(isize::MAX)));


read_and_write!(f32;
    (b"\x00\x00\x00\x00", Ok(0f32)),
    (b"\x40\x48\xF5\xC3", Ok(3.14f32)),
);

read_and_write!(f64;
    (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok(0f64)),
    (b"\x40\x09\x1E\xB8\x51\xEB\x85\x1F", Ok(3.14f64)),
);

read_and_write!(Vec<u8>;
    // Arrays are prefixed with an usize (4/8 bytes!)
    (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok(Vec::new())),
    (b"\x00\x00\x00\x00\x00\x00\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00", Ok(vec![0u8; 8])),
    (b"\x00\x00\x00\x00\x00\x00\x00\x04AAAA", Ok(vec![65u8, 65u8, 65u8, 65u8])),
);

read_and_write!(String;
    // The strings are treated as an array and thus prefixed with an usize (4/8 bytes!)
    (b"\x00\x00\x00\x00\x00\x00\x00\x00", Ok("".to_string())),
    (b"\x00\x00\x00\x00\x00\x00\x00\x04AAAA", Ok("AAAA".to_string())),
);

read_and_write!(Option<u8>;
    (b"\x00", Ok(None)),
    (b"\x01\x00", Ok(Some(0u8))),
    (b"\x01\xFF", Ok(Some(255u8))),
    (b"\x02", Err(Error::InvalidBoolean(2))),
);