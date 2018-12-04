//! Stage files in git using a selector
//!
//! ```sh
//! $ giadd
//! > [ ]  M src/main.rs
//!   [*] ?? a/new/file
//!   [ ] ?? another/new/file
//! ```
//!
//! # Keybinds
//! j and k to navigate  
//! q, escape to exit  
//! space to select a file  
//! enter to stage selected files  
//!

extern crate giadd;
extern crate libc;
extern crate termios;

use giadd::{
    add_selector, check_for_help_flag, clear_screen, display, fmt_files_to_strings, git_status,
    marshal_status_in_files, read_input, reset_terminal, set_terminal_to_raw,
};
use libc::STDIN_FILENO;
use std::process;
use termios::Termios;

fn main() {
    check_for_help_flag();

    let mut max_number_of_lines = 0;
    let mut selector_position = 0;
    let original_term = Termios::from_fd(STDIN_FILENO).unwrap();

    let git_status_output = git_status();

    if !git_status_output.status.success() {
        print!("{}", String::from_utf8_lossy(&git_status_output.stderr));
        match git_status_output.status.code() {
            Some(code) => process::exit(code),
            None => process::exit(1),
        };
    }

    let mut files = marshal_status_in_files(
        String::from_utf8(git_status_output.stdout).expect("Problem parsing status"),
    ).unwrap();

    loop {
        display(
            &mut max_number_of_lines,
            add_selector(selector_position, fmt_files_to_strings(&files)),
        );

        set_terminal_to_raw();

        if let Some(exit_code) = read_input(&mut selector_position, &mut files) {
            reset_terminal(&original_term);
            clear_screen(max_number_of_lines);
            process::exit(exit_code);
        } else {
            reset_terminal(&original_term);
        }
    }
}
