// Conversion between machine integers.

use std::{u8, u16, u32, u64, usize, i8, i16, i32, i64, isize};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::mem;

use {TryFrom, Void};

/// Error which occurs when conversion from one integer type to another fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TryFromIntError {
    Overflow,
    Underflow,
}

impl TryFromIntError {
    fn as_str(self) -> &'static str {
        match self {
            TryFromIntError::Overflow => "integer overflow",
            TryFromIntError::Underflow => "integer underflow",
        }
    }
}

impl Display for TryFromIntError {
    fn fmt(&self, n: &mut Formatter) -> fmt::Result {
        n.write_str(self.as_str())
    }
}

impl Error for TryFromIntError {
    fn description(&self) -> &str {
        self.as_str()
    }
}

macro_rules! impl_infallible {
    { $($t:ident from $($f:ident),*;)* } => { $($(
        impl TryFrom<$f> for $t {
            type Err = Void;
            fn try_from (n: $f) -> Result<$t, Void> { Ok(n as $t) }
        }
    )*)* };
}

impl_infallible! {
    u8 from u8;
    u16 from u8, u16;
    u32 from u8, u16, u32;
    u64 from u8, u16, u32, u64, usize;
    usize from u8, u16, u32, usize;
    i8 from i8;
    i16 from u8, i8, i16;
    i32 from u8, u16, i8, i16, i32;
    i64 from u8, u16, u32, i8, i16, i32, i64, isize;
    isize from u8, u16, i8, i16, i32, isize;
}

#[test]
fn test_infallible() {
    assert_eq!(u64::try_from(usize::MAX), Ok(usize::MAX as u64));
}

macro_rules! impl_unsigned_from_unsigned {
    { $($t:ident from $($f:ident),*;)* } => { $($(
        impl TryFrom<$f> for $t {
            type Err = TryFromIntError;

            fn try_from (n: $f) -> Result<$t, TryFromIntError> {
                if mem::size_of::<$f>() > mem::size_of::<$t>() && n > $t::MAX as $f {
                    Err(TryFromIntError::Overflow)
                } else {
                    Ok(n as $t)
                }
            }
        }
    )*)* };
}

impl_unsigned_from_unsigned! {
    u8 from u16, u32, u64, usize;
    u16 from u32, u64, usize;
    u32 from u64, usize;
    usize from u64;
}

#[test]
fn test_unsigned_from_unsigned() {
    assert_eq!(u8::try_from(0xffu16), Ok(0xffu8));
    assert_eq!(u8::try_from(0x100u16), Err(TryFromIntError::Overflow));

    if cfg!(target_pointer_width = "32") {
        assert_eq!(u32::try_from(usize::MAX), Ok(u32::MAX));
        assert_eq!(usize::try_from(u64::MAX), Err(TryFromIntError::Overflow));
    } else if cfg!(target_pointer_width = "64") {
        assert_eq!(u32::try_from(usize::MAX), Err(TryFromIntError::Overflow));
        assert_eq!(usize::try_from(u64::MAX), Ok(usize::MAX));
    }
}

macro_rules! impl_unsigned_from_signed {
    { $($t:ident from $($f:ident),*;)* } => { $($(
        impl TryFrom<$f> for $t {
            type Err = TryFromIntError;

            fn try_from (n: $f) -> Result<$t, TryFromIntError> {
                if n < 0 {
                    Err(TryFromIntError::Underflow)
                } else if mem::size_of::<$f>() > mem::size_of::<$t>() && n > $t::MAX as $f {
                    Err(TryFromIntError::Overflow)
                } else {
                    Ok(n as $t)
                }
            }
        }
    )*)* };
}

impl_unsigned_from_signed! {
    u8 from i8, i16, i32, i64, isize;
    u16 from i8, i16, i32, i64, isize;
    u32 from i8, i16, i32, i64, isize;
    u64 from i8, i16, i32, i64, isize;
    usize from i8, i16, i32, i64, isize;
}

#[test]
fn test_unsigned_from_signed() {
    assert_eq!(u8::try_from(0i16), Ok(0u8));
    assert_eq!(u8::try_from(-1i16), Err(TryFromIntError::Underflow));
    assert_eq!(u8::try_from(256i16), Err(TryFromIntError::Overflow));

    if cfg!(target_pointer_width = "32") {
        assert_eq!(u32::try_from(isize::MAX), Ok(0x7fff_ffffu32));
        assert_eq!(usize::try_from(i64::MAX), Err(TryFromIntError::Overflow));
    } else if cfg!(target_pointer_width = "64") {
        assert_eq!(u32::try_from(isize::MAX), Err(TryFromIntError::Overflow));
        assert!(usize::try_from(i64::MAX).unwrap() > 0xffff_ffffusize);
    }
}

macro_rules! impl_signed_from_unsigned {
    { $($t:ident from $($f:ident),*;)* } => { $($(
        impl TryFrom<$f> for $t {
            type Err = TryFromIntError;

            fn try_from (n: $f) -> Result<$t, TryFromIntError> {
                if mem::size_of::<$f>() >= mem::size_of::<$t>() && n > $t::MAX as $f {
                    Err(TryFromIntError::Overflow)
                } else {
                    Ok(n as $t)
                }
            }
        }
    )*)* };
}

impl_signed_from_unsigned! {
    i8 from u8, u16, u32, u64, usize;
    i16 from u16, u32, u64, usize;
    i32 from u32, u64, usize;
    i64 from u64, usize;
    isize from u32, u64, usize;
}

#[test]
fn test_signed_from_unsigned() {
    assert_eq!(i8::try_from(0x7fu8), Ok(0x7fi8));
    assert_eq!(i8::try_from(0x80u8), Err(TryFromIntError::Overflow));

    if cfg!(target_pointer_width = "32") {
        assert_eq!(i64::try_from(usize::MAX), Ok(0xffff_ffffi64));
        assert_eq!(
            isize::try_from(0x8000_0000u64),
            Err(TryFromIntError::Overflow)
        );
    } else if cfg!(target_pointer_width = "64") {
        assert_eq!(i64::try_from(usize::MAX), Err(TryFromIntError::Overflow));
        assert!(isize::try_from(0x8000_0000u64).unwrap() > 0x7fff_ffff);
    }
}

macro_rules! impl_signed_from_signed {
    { $($t:ident from $($f:ident),*;)* } => { $($(
        impl TryFrom<$f> for $t {
            type Err = TryFromIntError;

            fn try_from (n: $f) -> Result<$t, TryFromIntError> {
                if mem::size_of::<$f>() > mem::size_of::<$t>() {
                    if n > $t::MAX as $f {
                        return Err(TryFromIntError::Overflow);
                    } else if n < $t::MIN as $f {
                        return Err(TryFromIntError::Underflow);
                    }
                }
                Ok(n as $t)
            }
        }
    )*)* };
}

impl_signed_from_signed! {
    i8 from i16, i32, i64, isize;
    i16 from i32, i64, isize;
    i32 from i64, isize;
    isize from i64;
}

#[test]
fn test_signed_from_signed() {
    assert_eq!(i8::try_from(127i16), Ok(127i8));
    assert_eq!(i8::try_from(128i16), Err(TryFromIntError::Overflow));
    assert_eq!(i8::try_from(-128i16), Ok(-128i8));
    assert_eq!(i8::try_from(-129i16), Err(TryFromIntError::Underflow));

    if cfg!(target_pointer_width = "32") {
        assert_eq!(i32::try_from(isize::MAX), Ok(i32::MAX));
        assert_eq!(isize::try_from(i64::MAX), Err(TryFromIntError::Overflow));
    } else if cfg!(target_pointer_width = "64") {
        assert_eq!(i32::try_from(isize::MAX), Err(TryFromIntError::Overflow));
        assert!(isize::try_from(i64::MAX).unwrap() > 0x7fff_ffffisize);
    }
}
