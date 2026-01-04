use std::error::Error;
use std::io;
use std::time::Duration;
use std::io::{Read,Write};

use serialport::{self, ClearBuffer, DataBits, SerialPort, TTYPort};
// use serialport::posix::termios;

fn setup_serial() -> TTYPort{
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

/*
// FIXME: Can't get to serialport::posix::termios
// maybe use https://docs.rs/termios/latest/termios/
fn print_termios(port: &TTYPort) {
    let mut termios = termios::get_termios(port.as_raw_fd())?;
}
*/

fn print_serial_values(values: &[u8]) {
    // io::stdout().write_all(values).unwrap();
    for value in values {
        println!("{:#010b}", value);
    }
    io::stdout().flush().unwrap();
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut port = setup_serial();
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
                print_serial_values(slice);
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}

fn list_serial_ports() {
    let ports = serialport::available_ports().expect("No ports found!");
    println!("Available serial ports (according to libudev):");
    for p in ports {
        println!("    {}", p.port_name);
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
