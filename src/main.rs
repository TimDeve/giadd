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

use giadd::{check_for_help_flag, git_status, App};
use std::process;

fn main() {
    check_for_help_flag();

    let git_status_output = git_status();

    if !git_status_output.status.success() {
        print!("{}", String::from_utf8_lossy(&git_status_output.stderr));
        match git_status_output.status.code() {
            Some(code) => process::exit(code),
            None => process::exit(1),
        };
    }

    let mut app = App::new();

    match app.marshal_status_in_files(
        String::from_utf8(git_status_output.stdout).expect("Problem parsing status"),
    ) {
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
        Ok(()) => loop {
            let lines = app.fmt_files_to_strings();

            app.display(lines);

            app.set_terminal_to_raw();

            if let Some(exit_code) = app.read_input() {
                app.reset_terminal();
                app.clear_screen();
                process::exit(exit_code);
            } else {
                app.reset_terminal();
            }
        },
    };
}
