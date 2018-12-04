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
use terminal_size::{terminal_size, Width};
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

pub fn read_input(selector_position: &mut usize, files: &mut Vec<File>) -> Option<i32> {
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
            Keys::Space => select_file_under_selector(*selector_position, files),
            Keys::K => move_selector_up(selector_position, files.len()),
            Keys::J => move_selector_down(selector_position, files.len()),
            Keys::Enter => {
                let paths = get_selected_files_path(&files);
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
        }
    }
    return None;
}

pub fn check_for_help_flag() {
    if let Some(_) = env::args().find(|arg| arg == "--help" || arg == "-h") {
        display(
            &mut 0,
            vec![
                "giadd".to_string(),
                "Stage file in git using a selector".to_string(),
                "".to_string(),
                "KEYBINDS:".to_string(),
                "    j and k to navigate".to_string(),
                "    space to select a file".to_string(),
                "    enter to stage selected files".to_string(),
                "    q to exit".to_string(),
            ],
        );

        process::exit(0);
    }
}

fn select_file_under_selector(selector_position: usize, files: &mut Vec<File>) {
    files[selector_position].is_selected = !files[selector_position].is_selected
}

fn move_selector_down(selector_position: &mut usize, files_length: usize) {
    if *selector_position == files_length - 1 {
        *selector_position = 0;
    } else {
        *selector_position = *selector_position + 1;
    }
}

fn move_selector_up(selector_position: &mut usize, files_length: usize) {
    if *selector_position == 0 {
        *selector_position = files_length - 1;
    } else {
        *selector_position = *selector_position - 1;
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

pub fn marshal_status_in_files(status: String) -> Result<Vec<File>, &'static str> {
    let files: Result<Vec<File>, &str> = status
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
        }).collect();

    files
}

pub fn set_terminal_to_raw() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();
    cfmakeraw(&mut termios);
    tcsetattr(STDIN_FILENO, TCSANOW, &mut termios).unwrap();
}

pub fn reset_terminal(original_term: &Termios) {
    tcsetattr(STDIN_FILENO, TCSANOW, original_term).unwrap();
}

pub fn add_selector(selector_position: usize, lines: Vec<String>) -> Vec<String> {
    let mut new_lines: Vec<String> = vec![];

    for (i, line) in lines.iter().enumerate() {
        new_lines.push(format!(
            "{} {}",
            if selector_position == i { ">" } else { " " },
            line
        ))
    }

    new_lines
}

pub fn fmt_files_to_strings(files: &Vec<File>) -> Vec<String> {
    files
        .iter()
        .map(|file| {
            format!(
                "[{}] {} {}",
                if file.is_selected { "*" } else { " " },
                file.status,
                file.path
            )
        }).collect()
}

fn move_terminal_cursor_up(line: usize) {
    if line != 0 {
        print!("\x1b[{}A", line);
    }
}

pub fn clear_screen(line: usize) {
    move_terminal_cursor_up(line);

    let size = terminal_size();
    if let Some((Width(w), _)) = size {
        for _ in 0..line {
            println!("{}", " ".repeat(w as usize));
        }
        move_terminal_cursor_up(line);
    }
}

pub fn display(max_line_number: &mut usize, lines: Vec<String>) {
    clear_screen(*max_line_number);

    *max_line_number = lines.len();

    println!("{}", lines.join("\n"));
}

fn get_selected_files_path(files: &Vec<File>) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.is_selected)
        .map(|file| file.path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_status_into_files() {
        let status = String::from(" M src/main.rs\n?? wow\nCM src/wow.rs -> src/lib.rs");
        let files = marshal_status_in_files(status).unwrap();

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
            files
        )
    }

    #[test]
    fn returns_error_if_status_is_malformed() {
        let status = String::from(" M src/main.rs\n?? wow\nCM src/wow.rs ->");
        let error = marshal_status_in_files(status);

        assert_eq!(error, Err("Failed to parse status"))
    }

    #[test]
    fn files_to_strings() {
        let files = vec![
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

        assert_eq!(
            fmt_files_to_strings(&files),
            vec![
                String::from("[*] ?? /hello"),
                String::from("[ ]  M /is-it-me-you're-looking-for")
            ]
        )
    }

    #[test]
    fn add_selector_to_string() {
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];

        assert_eq!(
            add_selector(1, lines),
            vec!["  Line 1", "> Line 2", "  Line 3"]
        );
    }
}
