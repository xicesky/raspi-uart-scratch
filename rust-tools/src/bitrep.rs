use std::fmt::{self, Display};
use std::iter::Map;
use std::ops::Range;
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

/* FIXME: Leads to conflicting impl of trait
impl<B: MaybeBit> MaybeBit for &B {
    fn to_bit(&self) -> Option<bool> {
        (*self).to_bit()
    }
}
 */

pub trait PureBit {
    fn to_bool(&self) -> bool;
}

impl<PB: PureBit> PureBit for &PB {
    fn to_bool(&self) -> bool {
        (*self).to_bool()
    }
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

pub trait FromBits : Sized {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self;

    fn from_bits_msb<B: PureBit, T: IntoIterator<Item = B>>(iter: T) -> Self {
        Self::from_bits_iter(iter.into_iter())
    }

    fn from_bits_lsb<B: PureBit, T: IntoIterator<Item = B>>(iter: T) -> Self where
        T::IntoIter: DoubleEndedIterator
    {
        Self::from_bits_iter(iter.into_iter().rev())
    }

    fn from_maybebits_iter<B: MaybeBit, T: Iterator<Item = B>>(iter: T) -> Option<Self> {
        // FIXME: can we get by without an intermediate Vec<> ?
        let x: Option<Vec<_>> = iter
            .map(|b| b.to_bit())
            .collect();
        x.map(|vec| Self::from_bits_iter(vec.iter()))
    }

    fn from_maybebits_msb<B: MaybeBit, T: IntoIterator<Item = B>>(iter: T) -> Option<Self> {
        Self::from_maybebits_iter(iter.into_iter())
    }

    fn from_maybebits_lsb<B: MaybeBit, T: IntoIterator<Item = B>>(iter: T) -> Option<Self> where
        T::IntoIter: DoubleEndedIterator
    {
        Self::from_maybebits_iter(iter.into_iter().rev())
    }
}

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

impl MaybeBits for u16 {
    type BitElem = bool;
    fn bit_len(&self) -> usize {
        16
    }
    fn nth_bit(&self, n: usize) -> Self::BitElem {
        let x = *self & (0x1 << n);
        x != 0
    }
}

impl MaybeBits for u32 {
    type BitElem = bool;
    fn bit_len(&self) -> usize {
        32
    }
    fn nth_bit(&self, n: usize) -> Self::BitElem {
        let x = *self & (0x1 << n);
        x != 0
    }
}

impl MaybeBits for i8 {
    type BitElem = bool;
    fn bit_len(&self) -> usize {
        8
    }
    fn nth_bit(&self, n: usize) -> Self::BitElem {
        let x = *self & (0x1 << n);
        x != 0
    }
}

impl MaybeBits for i16 {
    type BitElem = bool;
    fn bit_len(&self) -> usize {
        16
    }
    fn nth_bit(&self, n: usize) -> Self::BitElem {
        let x = *self & (0x1 << n);
        x != 0
    }
}

impl MaybeBits for i32 {
    type BitElem = bool;
    fn bit_len(&self) -> usize {
        32
    }
    fn nth_bit(&self, n: usize) -> Self::BitElem {
        let x = *self & (0x1 << n);
        x != 0
    }
}

impl FixedLengthMaybeBits for u8 {
    const BIT_COUNT: usize = 8;
}

impl FromBits for u8 {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        let mut i: u8 = 0;
        for bit in iter {
            i <<= 1;
            i += if bit.to_bool() { 1 } else { 0 };
        }
        i
    }
}

impl FromBits for u16 {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        let mut i: u16 = 0;
        for bit in iter {
            i <<= 1;
            i += if bit.to_bool() { 1 } else { 0 };
        }
        i
    }
}

impl FromBits for u32 {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        let mut i: u32 = 0;
        for bit in iter {
            i <<= 1;
            i += if bit.to_bool() { 1 } else { 0 };
        }
        i
    }
}

impl FromBits for i8 {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        let mut i: i8 = 0;
        for bit in iter {
            i <<= 1;
            i += if bit.to_bool() { 1 } else { 0 };
        }
        i
    }
}

impl FromBits for i16 {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        let mut i: i16 = 0;
        for bit in iter {
            i <<= 1;
            i += if bit.to_bool() { 1 } else { 0 };
        }
        i
    }
}

impl FromBits for i32 {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        let mut i: i32 = 0;
        for bit in iter {
            i <<= 1;
            i += if bit.to_bool() { 1 } else { 0 };
        }
        i
    }
}

/***************************************************************************************************
 * "Bit" for signal decoding
 */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            Bit::Value(false)   => "0",
            Bit::Value(true)    => "1",
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

impl MaybeBit for &Bit {
    fn to_bit(&self) -> Option<bool> {
        match *self {
            Bit::Value(v) => Some(*v),
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

        #[test]
    fn test_frombits_i8() {
        let mut i: i8 = FromBits::from_bits_msb([true, false]);
        assert_eq!(i, 2);
        i = FromBits::from_bits_msb([true, false, false, false, false, false, false, false]);
        assert_eq!(i, -128);
    }

    #[test]
    fn test_frombits_u32() {
        let mut i: u32 = FromBits::from_bits_msb([true, false]);
        assert_eq!(i, 2);
        i = FromBits::from_bits_msb([true, false, true]);
        assert_eq!(i, 5);
    }
}
