// use std::error::Error;
use std::io;
use std::io::{Read,Write,Result};
use std::time::Duration;

use jiff::Zoned;
use num_traits::sign;
use serialport::{self, ClearBuffer, SerialPort};

use crate::dcf77_decoder::Error;

mod bitrep;
mod dcf77_decoder;
mod pulse_decoder;
mod raspi_refclock;

/*
// FIXME: Can't get to serialport::posix::termios
// maybe use https://docs.rs/termios/latest/termios/
fn print_termios(port: &TTYPort) {
    let mut termios = termios::get_termios(port.as_raw_fd())?;
}
*/

struct DebuggingDecoder {
    dcf_decoder: dcf77_decoder::Decoder,
    last_decoded: Option<Zoned>
}

impl DebuggingDecoder {
    fn new() -> DebuggingDecoder {
        DebuggingDecoder {
            dcf_decoder: dcf77_decoder::Decoder::new(),
            last_decoded: None
        }
    }

    fn handle_signal_byte(&mut self, signal: u8) {
        // io::stdout().write_all(values).unwrap();
        let bit = pulse_decoder::decode_pulse(signal ^ 0xFF);
        println!("Signal {:08b} = {}", signal, bit);
        self.dcf_decoder.add_bit(bit);
    }

    fn handle_signal_bytes(&mut self, signal: &[u8]) {
        for _ in 0..10 {
            println!();
        }
        for value in signal {
            self.handle_signal_byte(*value);
        }

        println!();
        println!("{:>60}", dcf77_decoder::DECODE_HEADER);
        println!("{:>60}", self.dcf_decoder);

        let mut current_error: Option<Error> = None;
        match self.dcf_decoder.decode_dcf77() {
            Ok(decoded) => self.last_decoded = Some(decoded),
            Err(e) => current_error = Some(e)
        }

        match current_error {
            Some(ref e) => println!("    last error: {}", e),
            None => println!("    last error: -")
        }

        match self.last_decoded {
            Some(ref time) => println!("dcf77:  {}", time),
            None => println!("dcf77: <no signal>")
        }

        io::stdout().flush().unwrap();
    }
}

fn run() -> Result<()> {
    let mut port = raspi_refclock::setup_serial();
    port.clear(ClearBuffer::Input)
        .expect("Failed to discard input buffer");

    // thread::sleep(Duration::from_millis(100));

    let mut serial_buf: Vec<u8> = vec![0; 1000];
    println!(
        "Receiving data on {} at {} baud:",
        port.name().unwrap_or(String::from("?")),
        port.baud_rate().map(|v| v.to_string()).unwrap_or(String::from("?"))
    );

    let mut decoder: DebuggingDecoder = DebuggingDecoder::new();

    loop {
        port.set_timeout(Duration::from_millis(1500))?;
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                println!("Read {} bytes", t);
                decoder.handle_signal_bytes(&serial_buf[..t]);
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                println!("Read timed out");
                decoder.handle_signal_bytes(&[0xFF]);
            },
            Err(e) => return Err(e),
        }
    }
}

fn main() {
    // TODO: Command line parsing
    // see e.g. https://github.com/serialport/serialport-rs/blob/main/examples/clear_input_buffer.rs

    // list_serial_ports();

    let exit_code = match run() {
        Ok(_) => 0,
        Err(e) => {
            println!("Error: {}", e);
            1
        }
    };

    std::process::exit(exit_code);

}
