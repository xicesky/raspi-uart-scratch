use std::fmt::{self, Display};
use std::iter::Map;
use std::slice::Iter;

/* Notes:
    * Maybe should be something like an iterator.
    * For most types, we can directly get bools, not just Option<bool>
        That should be supported somehow
    * Implement "Representaion" newtype that will print the bits
 */
pub trait MaybeBit {
    fn to_bit(&self) -> Option<bool>;
}

pub trait PureBit {
    fn to_bool(&self) -> bool;
}

impl<PB: PureBit> MaybeBit for PB {
    fn to_bit(&self) -> Option<bool> {
        Some(self.to_bool())
    }
}

/* Can't get this to work, don't understand the interaction of generics with lifetimes...
    Not enough info here?
    https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#generic-type-parameters-trait-bounds-and-lifetimes

trait IntoMaybeBits<'a> {
    type BitItem: 'a + MaybeBit;
    type IterType: Iterator<Item = &'a Self::BitItem>;
    // type
    // fn iter(&self) ->
    fn iter_bits(&self) -> Self::IterType;
}

impl<'a, BI: MaybeBit> IntoMaybeBits<'a> for Vec<BI> where
    BI: 'a,
    Self: 'a
{
    type BitItem = BI;
    type IterType = Iter<'a, BI>;
    fn iter_bits(&self) -> Self::IterType {
        return self.iter();
    }
}
 */

pub trait MaybeBits {
    type BitElem: MaybeBit + Copy;
    fn bit_len(&self) -> usize;
    fn nth_bit(&self, n: usize) -> Self::BitElem;

    fn to_bit_vector(&self) -> Vec<Self::BitElem> {
        let mut result = Vec::new();
        for i in 0..self.bit_len() {
            result.push(self.nth_bit(i));
        }
        result
    }
}

pub trait FixedLengthMaybeBits : MaybeBits {
    const BIT_COUNT: usize;
}

/*
impl<PB: FixedLengthMaybeBits> MaybeBits for PB {
    // Cannot implement partially :(
    fn bit_len(&self) -> usize {
        Self::BIT_COUNT
    }
}
*/

/***************************************************************************************************
 * Base impls
 */

impl PureBit for bool {
    fn to_bool(&self) -> bool {
        return *self;
    }
}

impl MaybeBits for u8 {
    type BitElem = bool;
    fn bit_len(&self) -> usize {
        8
    }
    fn nth_bit(&self, n: usize) -> Self::BitElem {
        let x = *self & (0x1 << n);
        x != 0
    }
}

impl FixedLengthMaybeBits for u8 {
    const BIT_COUNT: usize = 8;
}

/***************************************************************************************************
 * "Bit" for signal decoding
 */

#[derive(Clone, Copy, Debug)]
pub enum Bit {
    Unknown,
    Skipped,
    Value(bool)
}

impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match *self {
            Bit::Unknown        => "?",
            Bit::Skipped        => "_",
            Bit::Value(false)   => "|",
            Bit::Value(true)    => "#",
        })
    }
}

impl MaybeBit for Bit {
    fn to_bit(&self) -> Option<bool> {
        match *self {
            Bit::Value(v) => Some(v),
            _ => None
        }
    }
}

pub fn to_bit(b: impl MaybeBit) -> Bit {
    match b.to_bit() {
        Some(v) => Bit::Value(v),
        None => Bit::Unknown
    }
}
