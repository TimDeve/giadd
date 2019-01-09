#[macro_use]
extern crate enum_primitive;
extern crate libc;
extern crate num;
extern crate terminal_size;
extern crate termios;

use num::FromPrimitive;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::{io, process};
use terminal_size::{terminal_size, Height, Width};
use termios::{cfmakeraw, tcsetattr, Termios, TCSANOW};

pub fn select(lines: Vec<String>) -> Vec<String> {
    let mut state = SelectorState::new(lines);

    loop {
        let formatted_lines = state.fmt_lines_for_display();

        state.display(formatted_lines);

        state.set_terminal_to_raw();

        if let Some(strings) = state.read_input() {
            state.reset_terminal();
            state.clear_screen();
            return strings;
        } else {
            state.reset_terminal();
        }
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    enum Keys {
        CtrlC = 3,
        Enter = 13,
        Escape = 27,
        Space = 32,
        J = 106,
        K = 107,
        Q = 113,
    }
}

#[derive(Debug, PartialEq)]
struct Line {
    content: String,
    is_selected: bool,
}

struct SelectorState {
    max_number_of_lines: usize,
    selector_index: usize,
    top_of_screen_index: usize,
    lines: Vec<Line>,
    original_term: Termios,
    term: Termios,
    tty: File,
}

impl SelectorState {
    fn new(strings: Vec<String>) -> SelectorState {
        let tty = File::open("/dev/tty").unwrap();

        SelectorState {
            max_number_of_lines: 0,
            top_of_screen_index: 0,
            selector_index: 0,
            lines: marshal_strings_into_lines(strings),
            original_term: Termios::from_fd(tty.as_raw_fd()).unwrap(),
            term: Termios::from_fd(tty.as_raw_fd()).unwrap(),
            tty,
        }
    }

    fn read_input(&mut self) -> Option<Vec<String>> {
        let stdout = io::stdout();
        let mut buffer = [0; 3];

        stdout.lock().flush().unwrap();
        self.tty.read(&mut buffer).unwrap();

        if buffer[1] != 0 {
            return None;
        }

        if let Some(key) = Keys::from_u8(buffer[0]) {
            match key {
                Keys::Q | Keys::Escape => self.cleanup_and_exit(0),
                Keys::CtrlC => self.cleanup_and_exit(130),
                Keys::Space => self.select_file_under_selector(),
                Keys::K => self.move_selector_up(),
                Keys::J => self.move_selector_down(),
                Keys::Enter => return Some(self.get_selected_lines()),
            }
        }
        return None;
    }

    fn select_file_under_selector(&mut self) {
        self.lines[self.selector_index].is_selected = !self.lines[self.selector_index].is_selected
    }

    fn move_selector_down(&mut self) {
        if self.selector_index == self.lines.len() - 1 {
            self.selector_index = 0;
        } else {
            self.selector_index = self.selector_index + 1;
        }

        if self.selector_index > (self.top_of_screen_index + get_screen_height() - 2) {
            self.top_of_screen_index = self.top_of_screen_index + 1;
        }
    }

    fn move_selector_up(&mut self) {
        if self.selector_index == 0 {
            self.selector_index = self.lines.len() - 1;
        } else {
            self.selector_index = self.selector_index - 1;
        }

        if self.selector_index < self.top_of_screen_index {
            self.top_of_screen_index = self.top_of_screen_index - 1;
        }
    }

    fn get_selected_lines(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter(|line| line.is_selected)
            .map(|line| line.content.clone())
            .collect()
    }

    fn fmt_lines_for_display(&self) -> Vec<String> {
        let slice = &self.lines[..];

        let lines: Vec<String> = slice
            .iter()
            .enumerate()
            .map(|(i, line)| {
                format!(
                    "{} [{}] {}",
                    if self.selector_index == i { ">" } else { " " },
                    if line.is_selected { "*" } else { " " },
                    line.content
                )
            })
            .collect();

        if lines.len() > get_screen_height() {
            lines[self.top_of_screen_index..(self.top_of_screen_index + get_screen_height() - 1)]
                .to_vec()
        } else {
            lines
        }
    }

    fn clear_screen(&self) {
        move_terminal_cursor_up(self.max_number_of_lines);

        let size = terminal_size();
        if let Some((Width(w), _)) = size {
            for _ in 0..self.max_number_of_lines {
                println!("{}", " ".repeat(w as usize));
            }
            move_terminal_cursor_up(self.max_number_of_lines + 1);
            println!("");
        }
    }

    fn display(&mut self, lines: Vec<String>) {
        self.clear_screen();

        self.max_number_of_lines = lines.len();

        println!("{}", lines.join("\n"));
    }

    fn set_terminal_to_raw(&mut self) {
        cfmakeraw(&mut self.term);
        tcsetattr(self.tty.as_raw_fd(), TCSANOW, &mut self.term).unwrap();
    }

    fn reset_terminal(&self) {
        tcsetattr(self.tty.as_raw_fd(), TCSANOW, &self.original_term).unwrap();
    }

    fn cleanup_and_exit(&self, exit_code: i32) {
        self.reset_terminal();
        self.clear_screen();

        process::exit(exit_code);
    }
}

fn marshal_strings_into_lines(strings: Vec<String>) -> Vec<Line> {
    strings
        .iter()
        .map(|string| Line {
            content: string.to_string(),
            is_selected: false,
        })
        .collect()
}

fn get_screen_height() -> usize {
    let (_, Height(h)) = terminal_size().unwrap();

    h as usize
}

fn move_terminal_cursor_up(line: usize) {
    if line != 0 {
        print!("\x1b[{}A", line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_to_strings() {
        let mut g = SelectorState::new(vec![]);
        g.lines = vec![
            Line {
                content: "?? /hello".to_string(),
                is_selected: true,
            },
            Line {
                content: " M /is-it-me-you're-looking-for".to_string(),
                is_selected: false,
            },
        ];
        g.selector_index = 1;

        assert_eq!(
            g.fmt_lines_for_display(),
            vec![
                "  [*] ?? /hello".to_string(),
                "> [ ]  M /is-it-me-you're-looking-for".to_string(),
            ]
        )
    }
}
