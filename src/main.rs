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
extern crate selector;

use giadd::{check_for_help_flag, git_add, git_status, marshal_statuses_into_paths};
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

    let lines: Vec<String> = String::from_utf8(git_status_output.stdout)
        .expect("Problem parsing status")
        .lines()
        .map(|line| line.to_string())
        .collect();

    let selected_lines = selector::select(lines);
    let paths = marshal_statuses_into_paths(selected_lines).unwrap();

    let output = git_add(paths);

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        process::exit(0);
    } else {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        match output.status.code() {
            Some(code) => process::exit(code),
            None => process::exit(1),
        };
    }
}
