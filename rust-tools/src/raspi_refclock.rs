
use std::{io, vec};
use std::io::Write;
use std::time::Duration;
use serialport::{self, DataBits, TTYPort};
use ringbuffer::{RingBuffer,AllocRingBuffer};

pub fn list_serial_ports() {
    let ports = serialport::available_ports().expect("No ports found!");
    println!("Available serial ports (according to libudev):");
    for p in ports {
        println!("    {}", p.port_name);
    }
}

pub fn setup_serial() -> TTYPort{
    /* FIXME:
        See termios documentation: https://man7.org/linux/man-pages/man3/termios.3.html
        Termios flags set by
            term.c_cc[VMIN] = 1;                    // Special characters
            term.c_cflag = CS8|CREAD|CLOCAL|PARENB; // Control flags
            term.c_iflag = IGNPAR;                  // Input flags
            term.c_oflag = 0;                       // Output flags
            term.c_lflag = 0;                       // Local flags
        TTYPort::open sets:
            termios.c_cflag |= libc::CREAD | libc::CLOCAL;

     */
    let port = serialport::new("/dev/ttyAMA0", 50)
        .timeout(Duration::from_millis(100))
        // effectively sets c_cflag |= CS8
        .data_bits(DataBits::Eight)
        // effectively unsets c_cflag PARENB and PARODD
        // unsets c_iflag INPCK
        // sets c_iflag |= IGNPAR
        .parity(serialport::Parity::None)
        // not really neccessary
        // .dtr_on_open(false)
        .open_native()
        // .map_err(|ref e| format!("Port '{}' not available: {}", &port_name, e))?;
        .expect("Failed to open port");
    port
}

pub fn print_serial_values(values: &[u8]) {
    // io::stdout().write_all(values).unwrap();
    for value in values {
        println!("{:010b}", value);
    }
    io::stdout().flush().unwrap();
}
