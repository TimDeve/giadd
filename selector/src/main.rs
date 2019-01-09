extern crate selector;

use std::io;
use std::io::prelude::*;

fn main() {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .map(|line| line.unwrap().to_string())
        .collect();

    let selected_lines = selector::select(lines);

    if selected_lines.len() > 0 {
        println!("{}", selected_lines.join("\n"));
    }
}
