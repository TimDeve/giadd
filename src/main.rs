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

#[derive(Debug, PartialEq)]
struct File {
    status: String,
    path: String,
}

fn main() {
    ctrlc::set_handler(|| {
        set_terminal_to_cooked();
        process::exit(130);
    }).expect("Error setting ctrl-c handler");

    let output = get_git_status();

    if output.status.success() {
        let files = marshal_status_in_files(
            String::from_utf8(output.stdout).expect("Problem parsing status"),
        );

        println!("{:?}", files);

        set_terminal_to_rare();

        loop {
            read_key().unwrap();
        }
    } else {
        print!("{}", String::from_utf8_lossy(&output.stderr))
    }
}

fn get_git_status() -> process::Output {
    process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("failed to execute process")
}

fn marshal_status_in_files(status: String) -> Vec<File> {
    status
        .lines()
        .map(|line| File {
            status: line[0..2].to_string(),
            path: line[3..].to_string(),
        }).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_status_into_files() {
        let status = String::from(" M src/main.rs\n?? wow");
        let files = marshal_status_in_files(status);

        assert_eq!(
            vec![
                File {
                    status: String::from(" M"),
                    path: String::from("src/main.rs"),
                },
                File {
                    status: String::from("??"),
                    path: String::from("wow"),
                },
            ],
            files
        )
    }
}
