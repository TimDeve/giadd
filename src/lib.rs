#[macro_use]
extern crate enum_primitive;
extern crate libc;
extern crate num;
extern crate terminal_size;
extern crate termios;

use libc::STDIN_FILENO;
use num::FromPrimitive;
use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process;
use terminal_size::{terminal_size, Height, Width};
use termios::{cfmakeraw, tcsetattr, Termios, TCSANOW};

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    pub enum Keys {
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
pub struct File {
    status: String,
    path: String,
    is_selected: bool,
}

pub struct App {
    max_number_of_lines: usize,
    selector_position: usize,
    top_of_screen_position: usize,
    files: Vec<File>,
    original_term: Termios,
    term: Termios,
}

impl App {
    pub fn new() -> App {
        App {
            max_number_of_lines: 0,
            top_of_screen_position: 0,
            selector_position: 0,
            files: vec![],
            original_term: Termios::from_fd(STDIN_FILENO).unwrap(),
            term: Termios::from_fd(STDIN_FILENO).unwrap(),
        }
    }

    pub fn read_input(&mut self) -> Option<i32> {
        let stdout = io::stdout();
        let mut reader = io::stdin();
        let mut buffer = [0; 3];

        stdout.lock().flush().unwrap();
        reader.read(&mut buffer).unwrap();

        if buffer[1] != 0 {
            return None;
        }

        if let Some(key) = Keys::from_u8(buffer[0]) {
            match key {
                Keys::Q | Keys::Escape => return Some(0),
                Keys::CtrlC => return Some(130),
                Keys::Space => self.select_file_under_selector(),
                Keys::K => self.move_selector_up(),
                Keys::J => self.move_selector_down(),
                Keys::Enter => return self.add_selected_files(),
            }
        }
        return None;
    }

    fn add_selected_files(&self) -> Option<i32> {
        let paths = self.get_selected_files_path();
        let output = git_add(paths);

        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            return Some(0);
        } else {
            print!("{}", String::from_utf8_lossy(&output.stderr));
            return match output.status.code() {
                Some(code) => Some(code),
                None => Some(1),
            };
        }
    }

    fn select_file_under_selector(&mut self) {
        self.files[self.selector_position].is_selected =
            !self.files[self.selector_position].is_selected
    }

    fn move_selector_down(&mut self) {
        if self.selector_position == self.files.len() - 1 {
            self.selector_position = 0;
        } else {
            self.selector_position = self.selector_position + 1;
        }

        if self.selector_position > (self.top_of_screen_position + get_screen_height() - 2) {
            self.top_of_screen_position = self.top_of_screen_position + 1;
        }
    }

    fn move_selector_up(&mut self) {
        if self.selector_position == 0 {
            self.selector_position = self.files.len() - 1;
        } else {
            self.selector_position = self.selector_position - 1;
        }

        if self.selector_position < self.top_of_screen_position {
            self.top_of_screen_position = self.top_of_screen_position - 1;
        }
    }

    pub fn marshal_status_in_files(&mut self, status: String) -> Result<(), &'static str> {
        let file_result: Result<Vec<File>, &str> = status
            .lines()
            .map(|line| -> Result<File, &str> {
                let mut path = line[3..].to_string();

                if path.contains("->") {
                    path = match path.split_whitespace().nth(2) {
                        None => return Err("Failed to parse status"),
                        Some(p) => p.to_string(),
                    }
                }

                return Ok(File {
                    status: line[0..2].to_string(),
                    path,
                    is_selected: false,
                });
            })
            .collect();

        return match file_result {
            Ok(files) => {
                self.files = files;
                Ok(())
            }
            Err(e) => Err(e),
        };
    }

    pub fn set_terminal_to_raw(&mut self) {
        cfmakeraw(&mut self.term);
        tcsetattr(STDIN_FILENO, TCSANOW, &mut self.term).unwrap();
    }

    pub fn reset_terminal(&self) {
        tcsetattr(STDIN_FILENO, TCSANOW, &self.original_term).unwrap();
    }

    fn get_selected_files_path(&self) -> Vec<String> {
        self.files
            .iter()
            .filter(|file| file.is_selected)
            .map(|file| format!(":/{}", file.path))
            .collect()
    }

    pub fn fmt_files_to_strings(&self) -> Vec<String> {
        let slice = &self.files[..];

        let lines: Vec<String> = slice
            .iter()
            .map(|file| {
                format!(
                    "[{}] {} {}",
                    if file.is_selected { "*" } else { " " },
                    file.status,
                    file.path
                )
            })
            .collect();

        let mut new_lines: Vec<String> = vec![];

        for (i, line) in lines.iter().enumerate() {
            new_lines.push(format!(
                "{} {}",
                if self.selector_position == i {
                    ">"
                } else {
                    " "
                },
                line
            ))
        }

        if new_lines.len() > get_screen_height() {
            new_lines[self.top_of_screen_position
                ..(self.top_of_screen_position + get_screen_height() - 1)]
                .to_vec()
        } else {
            new_lines
        }
    }

    pub fn clear_screen(&self) {
        move_terminal_cursor_up(self.max_number_of_lines);

        let size = terminal_size();
        if let Some((Width(w), _)) = size {
            for _ in 0..self.max_number_of_lines {
                println!("{}", " ".repeat(w as usize));
            }
            move_terminal_cursor_up(self.max_number_of_lines);
        }
    }

    pub fn display(&mut self, lines: Vec<String>) {
        self.clear_screen();

        self.max_number_of_lines = lines.len();

        println!("{}", lines.join("\n"));
    }
}

pub fn get_screen_height() -> usize {
    let (_, Height(h)) = terminal_size().unwrap();

    h as usize
}

pub fn check_for_help_flag() {
    if let Some(_) = env::args().find(|arg| arg == "--help" || arg == "-h") {
        println!("giadd");
        println!("Stage file in git using a selector");
        println!("");
        println!("KEYBINDS:");
        println!("    j and k to navigate");
        println!("    space to select a file");
        println!("    enter to stage selected files");
        println!("    q to exit");

        process::exit(0);
    }
}

pub fn git_status() -> process::Output {
    process::Command::new("git")
        .arg("status")
        .arg("--porcelain=v1")
        .output()
        .expect("Failed to get git status")
}

fn git_add(paths: Vec<String>) -> process::Output {
    process::Command::new("git")
        .arg("add")
        .args(paths)
        .output()
        .expect("Failed to add files")
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
    fn turns_status_into_files() {
        let status = String::from(" M src/main.rs\n?? wow\nCM src/wow.rs -> src/lib.rs");

        let mut g = App::new();
        g.marshal_status_in_files(status).unwrap();

        assert_eq!(
            vec![
                File {
                    status: String::from(" M"),
                    path: String::from("src/main.rs"),
                    is_selected: false,
                },
                File {
                    status: String::from("??"),
                    path: String::from("wow"),
                    is_selected: false,
                },
                File {
                    status: String::from("CM"),
                    path: String::from("src/lib.rs"),
                    is_selected: false,
                },
            ],
            g.files
        )
    }

    #[test]
    fn returns_error_if_status_is_malformed() {
        let status = String::from(" M src/main.rs\n?? wow\nCM src/wow.rs ->");

        let mut g = App::new();
        let error = g.marshal_status_in_files(status);

        assert_eq!(error, Err("Failed to parse status"))
    }

    #[test]
    fn files_to_strings() {
        let mut g = App::new();
        g.files = vec![
            File {
                status: String::from("??"),
                path: String::from("/hello"),
                is_selected: true,
            },
            File {
                status: String::from(" M"),
                path: String::from("/is-it-me-you're-looking-for"),
                is_selected: false,
            },
        ];
        g.selector_position = 1;

        assert_eq!(
            g.fmt_files_to_strings(),
            vec![
                String::from("  [*] ?? /hello"),
                String::from("> [ ]  M /is-it-me-you're-looking-for")
            ]
        )
    }
}
