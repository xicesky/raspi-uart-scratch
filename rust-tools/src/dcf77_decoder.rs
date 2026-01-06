use std::fmt::{self, write};

use ringbuffer::{RingBuffer,AllocRingBuffer};
use serialport::Error;
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
const DECODE_HEADER : &str = "---------------RADMLS1248124P124812P1248121241248112481248P_";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecodingFailure {
    JustNotOk   // haha
}

pub type Result<T> = std::result::Result<T, DecodingFailure>;

struct Decoder {
    buffer: AllocRingBuffer<Bit>
    // let mut buffer = AllocRingBuffer::with_capacity(2);
}

impl fmt::Display for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.buffer.iter()
            .for_each(|x| { x.fmt(f); });
        Ok(())
    }
}

impl Decoder {
    fn new() -> Decoder {
        Decoder {
            buffer: AllocRingBuffer::new(60)
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn is_full(&self) -> bool {
        self.buffer.is_full()
    }

    fn add_maybe_bit<V: MaybeBit>(&mut self, value: V) -> &Self {
        self.buffer.enqueue(to_bit(value));
        self
    }

    fn add_bit(&mut self, value: Bit) -> &Self {
        self.buffer.enqueue(value);
        self
    }

    fn add_bits<V: MaybeBits>(&mut self, count: usize, value: V) -> &Self {
        assert!(count <= value.bit_len());
        let bitvec = value.to_bit_vector();
        for b in &bitvec[0..count] {
            self.add_maybe_bit(*b);
        }
        self
    }

    // FIXME: Implement indexing trait
    fn get_bit(&self, index: usize) -> Bit {
        assert!(index < 60);
        self.buffer[index]
    }

    fn decode_dcf77(&self) -> Result<bool> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_decoder() {
        let mut decoder: Decoder = Decoder::new();
        assert_eq!(decoder.len(), 0);

        // 0-10 Varied
        for _ in 0..11 {
            decoder.add_bit(Bit::Unknown);
        }
        assert_eq!(decoder.len(), 11);

        // 11-14 Free
        for _ in 11..15 {
            decoder.add_bit(Bit::Unknown);
        }
        assert_eq!(decoder.len(), 15);

        // 15,16 n.n.
        decoder.add_maybe_bit(false);
        decoder.add_maybe_bit(false);
        // 17,18 time zone
        decoder.add_maybe_bit(false);
        decoder.add_maybe_bit(true);
        // 19 n.n.
        decoder.add_maybe_bit(false);
        // 20 fix 1
        decoder.add_maybe_bit(true);
        assert_eq!(decoder.len(), 21);

        // 21-28 Minutes + Parity
        decoder.add_bits(4, 0);
        decoder.add_bits(3, 0);
        decoder.add_maybe_bit(false);
        assert_eq!(decoder.len(), 29);

        // 29-35 Hours + Parity
        decoder.add_bits(4, 0);
        decoder.add_bits(2, 0);
        decoder.add_maybe_bit(false);
        assert_eq!(decoder.len(), 36);

        // 36-41 Days
        decoder.add_bits(4, 1);
        decoder.add_bits(2, 0);
        assert_eq!(decoder.len(), 42);

        // 42-44 Day of week
        decoder.add_bits(3, 1 /* Monday */);
        assert_eq!(decoder.len(), 45);

        // 45-49 Month
        decoder.add_bits(4, 1);
        decoder.add_bits(1, 0);
        assert_eq!(decoder.len(), 50);

        // 50-57 Years
        decoder.add_bits(4, 6);
        decoder.add_bits(4, 1);
        assert_eq!(decoder.len(), 58);

        // 58 Date parity
        decoder.add_maybe_bit(false);
        assert_eq!(decoder.len(), 59);
        assert_eq!(decoder.decode_dcf77(), Ok(false));

        // 59 Missing
        decoder.add_bit(Bit::Skipped);
        assert_eq!(decoder.len(), 60);
        assert_eq!(decoder.decode_dcf77(), Ok(false));

        assert_eq!(decoder.is_full(), true);
        println!("{}", DECODE_HEADER);
        println!("{}", decoder);
        // for i in 0..60 {
        //     println!("    bit {:04}    {:?}", i, decoder.get_bit(i));
        // }
    }
}
