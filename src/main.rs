#[macro_use]
extern crate enum_primitive;
extern crate libc;
extern crate num;
extern crate terminal_size;
extern crate termios;

use libc::STDIN_FILENO;
use num::FromPrimitive;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process;
use terminal_size::{terminal_size, Width};
use termios::{cfmakeraw, tcsetattr, Termios, TCSANOW};

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
struct File {
    status: String,
    path: String,
    is_selected: bool,
}

fn main() {
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
    );

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

fn read_input(selector_position: &mut usize, files: &mut Vec<File>) -> Option<i32> {
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

fn select_file_under_selector(selector_position: usize, files: &mut Vec<File>) {
    files[selector_position].is_selected = !files[selector_position].is_selected
}

fn move_selector_down(selector_position: &mut usize, files_lenght: usize) {
    if *selector_position == files_lenght - 1 {
        *selector_position = 0;
    } else {
        *selector_position = *selector_position + 1;
    }
}

fn move_selector_up(selector_position: &mut usize, files_lenght: usize) {
    if *selector_position == 0 {
        *selector_position = files_lenght - 1;
    } else {
        *selector_position = *selector_position - 1;
    }
}

fn git_status() -> process::Output {
    process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
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

fn marshal_status_in_files(status: String) -> Vec<File> {
    status
        .lines()
        .map(|line| File {
            status: line[0..2].to_string(),
            path: line[3..].to_string(),
            is_selected: false,
        }).collect()
}

fn set_terminal_to_raw() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();
    cfmakeraw(&mut termios);
    tcsetattr(STDIN_FILENO, TCSANOW, &mut termios).unwrap();
}

fn reset_terminal(original_term: &Termios) {
    tcsetattr(STDIN_FILENO, TCSANOW, original_term).unwrap();
}

fn add_selector(selector_position: usize, lines: Vec<String>) -> Vec<String> {
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

fn fmt_files_to_strings(files: &Vec<File>) -> Vec<String> {
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

fn clear_screen(line: usize) {
    move_terminal_cursor_up(line);

    let size = terminal_size();
    if let Some((Width(w), _)) = size {
        for _ in 0..line {
            println!("{}", " ".repeat(w as usize));
        }
        move_terminal_cursor_up(line);
    }
}

fn display(max_line_number: &mut usize, lines: Vec<String>) {
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
        let status = String::from(" M src/main.rs\n?? wow");
        let files = marshal_status_in_files(status);

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
            ],
            files
        )
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
