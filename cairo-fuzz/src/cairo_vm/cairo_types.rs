use serde::{Deserialize, Serialize, Serializer};

pub trait Unsigned: Copy + Default + 'static {
    const U8: u8;
    const U16: u16;
    const U32: u32;
    const U64: u64;
    const USIZE: usize;
    const I8: i8;
    const I16: i16;
    const I32: i32;
    const I64: i64;
    const ISIZE: isize;

    fn to_u8() -> u8;
    fn to_u16() -> u16;
    fn to_u32() -> u32;
    fn to_u64() -> u64;
    fn to_usize() -> usize;
    fn to_i8() -> i8;
    fn to_i16() -> i16;
    fn to_i32() -> i32;
    fn to_i64() -> i64;
    fn to_isize() -> isize;
}
pub trait Bit: Copy + Default + 'static {
    const U8: u8;
    const BOOL: bool;

    fn new() -> Self;
    fn to_u8() -> u8;
    fn to_bool() -> bool;
}

/// The type-level bit 0.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default, Serialize, Deserialize)]
pub struct B0;

impl B0 {
    /// Instantiates a singleton representing this bit.
    #[inline]
    pub fn new() -> B0 {
        B0
    }
}

/// The type-level bit 1.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default, Serialize, Deserialize)]
pub struct B1;

impl B1 {
    /// Instantiates a singleton representing this bit.
    #[inline]
    pub fn new() -> B1 {
        B1
    }
}

impl Bit for B0 {
    const U8: u8 = 0;
    const BOOL: bool = false;

    #[inline]
    fn new() -> Self {
        Self
    }
    #[inline]
    fn to_u8() -> u8 {
        0
    }
    #[inline]
    fn to_bool() -> bool {
        false
    }
}

impl Bit for B1 {
    const U8: u8 = 1;
    const BOOL: bool = true;

    #[inline]
    fn new() -> Self {
        Self
    }
    #[inline]
    fn to_u8() -> u8 {
        1
    }
    #[inline]
    fn to_bool() -> bool {
        true
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default, Serialize, Deserialize)]
pub struct UTerm;

impl UTerm {
    /// Instantiates a singleton representing this unsigned integer.
    #[inline]
    pub fn new() -> UTerm {
        UTerm
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default, Serialize, Deserialize)]
pub struct UInt<U, B> {
    /// The more significant bits of `Self`.
    pub(crate) msb: U,
    /// The least significant bit of `Self`.
    pub(crate) lsb: B,
}

impl<U: Unsigned, B: Bit> UInt<U, B> {
    /// Instantiates a singleton representing this unsigned integer.
    #[inline]
    pub fn new() -> UInt<U, B> {
        UInt::default()
    }
}

pub type Felt = UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B0>;


#[derive(Debug, Clone,  Serialize, Deserialize)]
pub enum CairoTypes {
    Felt(Felt),
}
