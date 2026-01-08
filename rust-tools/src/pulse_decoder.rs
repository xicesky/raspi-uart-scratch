use crate::bitrep::Bit;

fn decode_pulse(pulse: u8) -> Bit {
    // Check that "pulsed" is a series of ones starting from lsb
    if ((pulse + 1) & pulse >= 1) {
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
