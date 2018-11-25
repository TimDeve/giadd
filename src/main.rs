#[macro_use]
extern crate enum_primitive;
extern crate ctrlc;
extern crate libc;
extern crate num;
extern crate termios;

use libc::STDIN_FILENO;
use num::FromPrimitive;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    enum Keys {
        Enter = 10,
        Space = 32,
        J = 106,
        K = 107,
        Q = 113,
    }
}

fn main() {
    ctrlc::set_handler(|| {
        set_terminal_to_cooked();
        process::exit(130);
    }).expect("Error setting ctrl-c handler");

    set_terminal_to_rare();

    loop {
        read_key().unwrap();
    }
}

fn set_terminal_to_rare() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();
    termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &mut termios).unwrap();
}

fn set_terminal_to_cooked() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();
    termios.c_lflag |= !(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &mut termios).unwrap();
}

fn read_key() -> io::Result<()> {
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0; 1];

    stdout.lock().flush()?;
    reader.read_exact(&mut buffer)?;

    if let Some(key) = Keys::from_u8(buffer[0]) {
        match key {
            Keys::J => println!("J"),
            Keys::K => println!("K"),
            Keys::Q => {
                set_terminal_to_cooked();
                process::exit(0);
            }
            Keys::Enter => println!("Enter"),
            Keys::Space => println!("Space"),
        }
    }

    Ok(())
}
