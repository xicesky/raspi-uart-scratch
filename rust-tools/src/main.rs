// use std::error::Error;
use std::io;
use std::io::{Read,Write,Result};

use serialport::{self, ClearBuffer, SerialPort};

mod bitrep;
mod dcf77_decoder;
mod raspi_refclock;

/*
// FIXME: Can't get to serialport::posix::termios
// maybe use https://docs.rs/termios/latest/termios/
fn print_termios(port: &TTYPort) {
    let mut termios = termios::get_termios(port.as_raw_fd())?;
}
*/

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

    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                println!("Read {} bytes", t);
                let slice = &serial_buf[..t];
                raspi_refclock::print_serial_values(slice);
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
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
