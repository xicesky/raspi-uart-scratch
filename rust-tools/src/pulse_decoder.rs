use crate::bitrep::Bit;

pub fn decode_pulse(pulse: u8) -> Bit {
    // Check that "pulsed" is a series of ones starting from lsb
    if pulse.wrapping_add(1) & pulse >= 1 {
        return Bit::Unknown
    } else if pulse == 0 {
        return Bit::Skipped
    } else {
        /* We assume 50baud, i.e. 1 bit represents an interval of 20ms
            The first 20ms are "consumed" by the UART as the start bit,
            so the pulse length = 20ms * (count(bits) + 1).
            We want to check if the pulse is longer than 100ms, so we check
                100ms < 20ms * (count(bits) + 1)
            <=>     4 < count(bits)
         */
        return Bit::Value(pulse > 0xF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        assert_eq!(decode_pulse(0b0), Bit::Skipped);
        assert_eq!(decode_pulse(0b1), Bit::Value(false));
        assert_eq!(decode_pulse(0b10), Bit::Unknown);
        assert_eq!(decode_pulse(0b11), Bit::Value(false));
        assert_eq!(decode_pulse(0b100), Bit::Unknown);
        assert_eq!(decode_pulse(0b101), Bit::Unknown);
        assert_eq!(decode_pulse(0b110), Bit::Unknown);
        assert_eq!(decode_pulse(0b111), Bit::Value(false));
        assert_eq!(decode_pulse(0b1111), Bit::Value(false));
        assert_eq!(decode_pulse(0b11111), Bit::Value(true));
        assert_eq!(decode_pulse(0b111110), Bit::Unknown);
        assert_eq!(decode_pulse(0b111111), Bit::Value(true));
        assert_eq!(decode_pulse(0xFF), Bit::Value(true));
    }
}
