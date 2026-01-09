use std::fmt::{self};

use jiff::{Zoned, civil::DateTime, tz};
use num_traits::NumCast;
use ringbuffer::{RingBuffer,AllocRingBuffer};
// use serialport::Error;

use crate::bitrep::*;

/* Encoding is:
 * Second       Contents
 * 0  - 10      AM: free, FM: 0
 * 11 - 14      free
 * 15           R     - "call bit" used to signalize irregularities in the control facilities
 * 16           A1    - expect zone change (1 hour before)
 * 17 - 18      Z1,Z2 - time zone
 *       0      0 illegal
 *       0      1 MEZ  (MET)
 *       1      0 MESZ (MED, MET DST)
 *       1      1 illegal
 * 19           A2    - expect leap insertion/deletion (1 hour before)
 * 20           S     - start of time code (1)
 * 21 - 24      M1    - BCD (lsb first) Minutes
 * 25 - 27      M10   - BCD (lsb first) 10 Minutes
 * 28           P1    - Minute Parity (even)
 * 29 - 32      H1    - BCD (lsb first) Hours
 * 33 - 34      H10   - BCD (lsb first) 10 Hours
 * 35           P2    - Hour Parity (even)
 * 36 - 39      D1    - BCD (lsb first) Days
 * 40 - 41      D10   - BCD (lsb first) 10 Days
 * 42 - 44      DW    - BCD (lsb first) day of week (1: Monday -> 7: Sunday)
 *  v----------- Note: Here the comment in NTP code is WRONG (month uses 4+1 bits, not 5+1)
 * 45 - 48      MO    - BCD (lsb first) Month
 * 49           MO0   - 10 Months
 * 50 - 53      Y1    - BCD (lsb first) Years
 * 54 - 57      Y10   - BCD (lsb first) 10 Years
 * 58           P3    - Date Parity (even)
 * 59                 - usually missing (minute indication), except for leap insertion
 */
pub const DECODE_HEADER : &str = "---------------RADMLS1248124P124812P1248121241248112481248P_";

/* Macro for error testing, borrowed from the "matches" crate:
    https://docs.rs/matches/0.1.10/matches/macro.assert_matches.html
    Only used for tests
*/
#[cfg(test)]
macro_rules! assert_matches {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => (),
            ref e => panic!("assertion failed: `{:?}` does not match `{}`", e, stringify!($($pattern)+)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParityBitName {
    Minute,
    Hour,
    Date
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodingFailure {
    NotEnoughBits,
    MissingBit,
    ParityError(ParityBitName),
    MissingStartOfTimeCode,
    NotSync,    /* Missing "skipped" bit 59 */
    InvalidTimezoneBits,
    BCDNotBigenough
}

#[derive(Clone, Debug)]
pub enum Error {
    DecodingError(DecodingFailure),
    JiffError(jiff::Error)
}

#[allow(unused)]
impl Error {
    fn decoding_failure(&self) -> Option<DecodingFailure> {
        match self {
            Self::DecodingError(e) => Some(*e),
            _ => None
        }
    }

    fn jiff_error(&self) -> Option<jiff::Error> {
        match self {
            Self::JiffError(e) => Some(e.clone()),
            _ => None
        }
    }
}

/* TODO: Traits for error (see e.g. jiff via in_tz())
    impl std::error::Error for Error {}
    impl core::fmt::Display for Error
    impl core::fmt::Debug for Error
 */

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // FIXME
        write!(f, "{:?}", self)
    }
}

impl From<DecodingFailure> for Error {
    fn from(value: DecodingFailure) -> Self {
        Error::DecodingError(value)
    }
}

impl From<jiff::Error> for Error {
    fn from(value: jiff::Error) -> Self {
        Error::JiffError(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<DecodingFailure> for Result<T> {
    fn from(value: DecodingFailure) -> Self {
        Err(From::from(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dcf77Tz {
    MEZ,
    MESZ
}

impl Dcf77Tz {
    fn to_utc_offset(&self) -> tz::Offset {
        match *self {
            Self::MEZ => tz::offset(1),
            Self::MESZ => tz::offset(2),
        }
    }
    fn to_time_zone(&self) -> tz::TimeZone {
        self.to_utc_offset().to_time_zone()
    }
}

impl std::fmt::Display for Dcf77Tz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromBits for Result<Dcf77Tz> {
    fn from_bits_iter<B: PureBit, T: Iterator<Item = B>>(iter: T) -> Self {
        /* We read everything as LSB, so this method will also be called with
            an already reversed iterator. Thus using from_bits_iter for another
            type (here: u8) will return an int with LSB order.
        */
        let v: u8 = FromBits::from_bits_iter(iter);
        match v {
            0b10 /* Bits: zero, one */ => Ok(Dcf77Tz::MEZ),
            0b01 /* Bits: one, zero */ => Ok(Dcf77Tz::MESZ),
            _    => From::from(DecodingFailure::InvalidTimezoneBits)
        }
    }
}

fn check_parity(name: ParityBitName, bits: &[Bit]) -> Result<()> {
    let mut parity = false; /* even */
    for bit in bits {
        let value = bit.to_bit()
            .ok_or(DecodingFailure::MissingBit)?; // Why does this work, but not (1)
        parity ^= value;
    }
    // FIXME: simplify! how?
    if parity { From::from(DecodingFailure::ParityError(name)) }
    else { Ok(()) }
}

fn decode_bits<T>(bits: &[Bit]) -> Result<T> where
    T: FromBits
{
    FromBits::from_maybebits_lsb(bits)
        .ok_or(From::from(DecodingFailure::MissingBit)) // (1)
}

fn decode_bcd<T>(lower_bits: &[Bit], higher_bits: &[Bit]) -> Result<T> where
    T: FromBits + num_traits::PrimInt
{
    let ten: T = NumCast::from(10)
        .ok_or(DecodingFailure::BCDNotBigenough)?;
    let lower: T = decode_bits(lower_bits)?;
    let higher: T = decode_bits(higher_bits)?;
    Ok(higher * ten + lower)
}

pub struct Decoder {
    buffer: AllocRingBuffer<Bit>
    // let mut buffer = AllocRingBuffer::with_capacity(2);
}

impl fmt::Display for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in self.buffer.iter() {
            write!(f, "{}", x)?;
        }
        Ok(())
    }
}

#[allow(unused)]
impl Decoder {
    pub fn new() -> Decoder {
        Decoder {
            buffer: AllocRingBuffer::new(60)
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        self.buffer.is_full()
    }

    // FIXME: Remove?
    pub fn to_vec(&self) -> Vec<Bit> {
        self.buffer.to_vec()
    }

    pub fn add_maybe_bit<V: MaybeBit>(&mut self, value: V) -> &Self {
        self.buffer.enqueue(to_bit(value));
        self
    }

    pub fn add_bit(&mut self, value: Bit) -> &Self {
        self.buffer.enqueue(value);
        self
    }

    pub fn add_bits<B, V>(&mut self, count: usize, value: V) -> &Self where
        B: MaybeBit + Copy,
        V: MaybeBits<BitElem = B>
    {
        assert!(count <= value.bit_len());
        let bitvec = value.to_bit_vector();
        let bits = (&bitvec[0..count]).iter().map(|b| to_bit(*b));
        self.add_bit_iter(bits);
        self
    }

    pub fn add_bitvec(&mut self, count: usize, bitvec: &Vec<Bit>) -> &Self {
        assert!(count <= bitvec.len());
        self.add_bit_iter(bitvec[0..count].iter().map(|b| *b));
        self
    }

    pub fn add_bit_iter(&mut self, iter: impl IntoIterator<Item = Bit>) -> &Self {
        self.buffer.extend(iter);
        self
    }

    pub fn add_bit_ref_iter<'a>(&mut self, iter: impl IntoIterator<Item = &'a Bit>) -> &Self {
        self.buffer.extend(iter.into_iter().copied());
        self
    }

    // FIXME: Implement indexing trait
    pub fn get_bit(&self, index: usize) -> Bit {
        assert!(index < 60);
        self.buffer[index]
    }

    // fn decode_bits(&self, r: Range<usize>) {
    //     let vs = &(self.buffer.to_vec()[r]);

    // }

    pub fn decode_dcf77(&self) -> Result<Zoned> {
        if !self.buffer.is_full() {
            return From::from(DecodingFailure::NotEnoughBits)
        }
        let bitvec = self.buffer.to_vec();

        // "Sync": Bit 59 should be skipped
        if bitvec[59] != Bit::Skipped {
            return From::from(DecodingFailure::NotSync)
        }

        let dcf77_tz_res: Result<Dcf77Tz> = decode_bits(&bitvec[17..19])?;
        let dcf77_tz: Dcf77Tz = dcf77_tz_res?;

        if bitvec[20] != Bit::Value(true) {
            return From::from(DecodingFailure::MissingStartOfTimeCode)
        }
        check_parity(ParityBitName::Minute, &bitvec[21..29])?;
        let minute: i8 = decode_bcd(&bitvec[21..25], &bitvec[25..28])?;
        check_parity(ParityBitName::Hour, &bitvec[29..36])?;
        let hour: i8 = decode_bcd(&bitvec[29..33], &bitvec[33..35])?;
        check_parity(ParityBitName::Date, &bitvec[36..59])?;
        let day: i8 = decode_bcd(&bitvec[36..40], &bitvec[40..42])?;
        let month: i8 = decode_bcd(&bitvec[45..49], &bitvec[49..50])?;
        let year: i8 = decode_bcd(&bitvec[50..54], &bitvec[54..58])?;

        let full_year: i16 = 2000 + year as i16;

        /* jiff date() / at() / in_tz() args:
            year: i16,
            month: i8,
            day: i8
            hour: i8,
            minute: i8,
            second: i8,
            subsec_nanosecond: i32,
            time_zone_name: &str
         */
        let dt = DateTime::new(
            full_year, month, day,
            hour, minute, 0, 0
        );
        let tz = dcf77_tz.to_time_zone();
        match dt {
            Ok(value) => {
                let x = value.to_zoned(tz)?;
                Ok(x)
            }
            // Err(e) if e.is_range() => {
            //     Err(From::from(DecodingFailure::ParityError))
            // }
            Err(e) => Err(From::from(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use jiff::{Unit, Zoned};
    use super::*;

    fn add_bits_helper<B, V>(rb: &mut AllocRingBuffer<Bit>, count: usize, value: V) where
        B: MaybeBit + Copy,
        V: MaybeBits<BitElem = B>
    {
        assert!(count <= value.bit_len());
        let bitvec = value.to_bit_vector();
        let bits = (&bitvec[0..count]).iter().map(|b| to_bit(*b));
        rb.extend(bits);
    }

    fn build_valid_signal() -> Vec<Bit> {
        let mut buffer: AllocRingBuffer<Bit> = AllocRingBuffer::new(60);

        // 0-10 Varied
        for _ in 0..11 {
            buffer.enqueue(Bit::Unknown);
        }

        // 11-14 Free
        for _ in 11..15 {
            buffer.enqueue(Bit::Unknown);
        }

        // 15,16 n.n.
        buffer.enqueue(to_bit(false));
        buffer.enqueue(to_bit(false));
        // 17,18 time zone
        buffer.enqueue(to_bit(false));
        buffer.enqueue(to_bit(true));
        // 19 n.n.
        buffer.enqueue(to_bit(false));
        // 20 fix 1
        buffer.enqueue(to_bit(true));

        // 21-28 Minutes + Parity
        add_bits_helper(&mut buffer, 4, 0);
        add_bits_helper(&mut buffer, 3, 0);
        buffer.enqueue(to_bit(false));

        // 29-35 Hours + Parity
        add_bits_helper(&mut buffer, 4, 0);
        add_bits_helper(&mut buffer, 2, 0);
        buffer.enqueue(to_bit(false));

        // 36-41 Days
        add_bits_helper(&mut buffer, 4, 1);
        add_bits_helper(&mut buffer, 2, 0);

        // 42-44 Day of week
        add_bits_helper(&mut buffer, 3, 1 /* Monday */);

        // 45-49 Month
        add_bits_helper(&mut buffer, 4, 1);
        add_bits_helper(&mut buffer, 1, 0);

        // 50-57 Years
        add_bits_helper(&mut buffer, 4, 6);
        add_bits_helper(&mut buffer, 4, 1);

        // 58 Date parity
        buffer.enqueue(to_bit(false));

        // 59 Missing
        buffer.enqueue(Bit::Skipped);
        assert_eq!(buffer.len(), 60);

        buffer.to_vec()
    }

    #[test]
    fn test_decoder_valid_signal() {
        let signal = build_valid_signal();
        let mut decoder: Decoder = Decoder::new();
        assert_eq!(decoder.len(), 0);

        decoder.add_bit_ref_iter(signal[0..59].iter());
        assert_eq!(decoder.len(), 59);

        // Check that this does not decode
        assert_eq!(decoder.is_full(), false);
        let decoded = decoder.decode_dcf77();
        assert_matches!(decoded, Err(Error::DecodingError(DecodingFailure::NotEnoughBits)));
        // assert_matches!(decoded, Err(Error::DecodingError(DecodingFailure::NotSync)));

        println!();
        /* FIXME: right alignment doesn't work. This seems to be general rust problem:
            https://users.rust-lang.org/t/should-display-implementations-respect-width-fill-align/110476
         */
        println!();
        println!("{:>60}", DECODE_HEADER);
        println!("{:>60}", decoder);

        // 59 Missing
        decoder.add_bit(Bit::Skipped);
        assert_eq!(decoder.len(), 60);
        // assert_eq!(decoder.decode_dcf77(), Ok(false));

        assert_eq!(decoder.is_full(), true);
        println!();
        println!("{:>60}", DECODE_HEADER);
        println!("{:>60}", decoder);
        // for i in 0..60 {
        //     println!("    bit {:04}    {:?}", i, decoder.get_bit(i));
        // }

        let now = Zoned::now().round(Unit::Second)
            .expect("Could not get current time");
        println!("now:    {}", now);
        let decoded = decoder.decode_dcf77()
            .expect("Could not decode dcf77 time");
        println!("dcf77:  {}", decoded);

        // Adding more bits will desync
        decoder.add_bit(Bit::Unknown);
        let decoded = decoder.decode_dcf77();
        assert_matches!(decoded, Err(Error::DecodingError(DecodingFailure::NotSync)));
    }

    #[test]
    fn test_decoder_do_not_panic() {
        // The decoder should not panic if fields are out of range
        let mut signal = build_valid_signal();
        signal[21..29].copy_from_slice(&[
            // 1101 (11, lsb)
            Bit::Value(true),
            Bit::Value(true),
            Bit::Value(false),
            Bit::Value(true),
            // 101 (5, lsb)
            Bit::Value(true),
            Bit::Value(false),
            Bit::Value(true),
            // 1 (even parity)
            Bit::Value(true),
        ]);
        let mut decoder: Decoder = Decoder::new();

        decoder.add_bit_ref_iter(signal.iter());
        assert_eq!(decoder.len(), 60);

        let decoded = decoder.decode_dcf77();
        println!("{:?}", decoded);
        assert_matches!(decoded, Err(Error::JiffError(_)));
    }

    /* TODO: Use some real-life examples:
        ---------------RADMLS1248124P124812P1248121241248112481248P_
        01001011111101100010101100101011010110010111110000011001010_
            last error: DecodingError(ParityError)

        ---------------RADMLS1248124P124812P1248121241248112481248P_
        01101100100001100010111100100011010110010010110000011101001_
            last error: -
        dcf77:  2034-01-09T16:27:00+02:00[+02:00]

        FIXME: MESZ !? Nope, that's wrong

        ---------------RADMLS1248124P124812P1248121241248112481248P_
        01011110111101100010111100100000110010010010110000011001010_
            last error: DecodingError(ParityError(Date))

        0         1         2         3         4         5
        012345678901234567890123456789012345678901234567890123456789
        ---------------RADMLS1248124P124812P1248121241248112481248P_
        00110100010000100010110101100000110010110110110000011001000_
            last error: JiffError(parameter 'day' with value 33 is not in the required range of 1..=31)

        1248 12 124 1248 1 1248 1248 P_
        1011 01 101 1000 0 0110 0100 0_
        13   2  5   1    0 6    2

        ============================================================================================
        Bug!? It's 2026-01-10 02:04 (or sth)

        0         1         2         3         4         5
        012345678901234567890123456789012345678901234567890123456789
        ---------------RADMLS1248124P124812P1248121241248112481248P_
        00001110011100100010110100001010100101011011111001011001001_
            last error: DecodingError(ParityError(Minute))
        dcf77:  2026-01-10T19:49:00+02:00[+02:00]

        Time bits:
        2             3
        0 1234 567 8 9012 34 5
        S 1248 124 P 1248 12 P
        1 1010 000 1 0101 00 1
        # 5    0   ! 12   0  !

     */
}
