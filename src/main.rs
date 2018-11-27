#[macro_use]
extern crate enum_primitive;
extern crate libc;
extern crate num;
extern crate termios;

use libc::STDIN_FILENO;
use num::FromPrimitive;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process;
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
    let mut max_line_number = 0;
    let mut cursor_position = 0;
    let original_termios = Termios::from_fd(STDIN_FILENO).unwrap();

    let output = get_git_status();

    if output.status.success() {
        let mut files = marshal_status_in_files(
            String::from_utf8(output.stdout).expect("Problem parsing status"),
        );

        loop {
            display(
                &mut max_line_number,
                add_cursor(cursor_position, fmt_files_to_strings(&files)),
            );
            println!("");
            set_terminal_to_raw();
            read_key(&original_termios, &mut cursor_position, &mut files).unwrap();
            reset_terminal(&original_termios);
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

fn reset_terminal(original_termios: &Termios) {
    tcsetattr(STDIN_FILENO, TCSANOW, original_termios).unwrap();
}

fn add_cursor(cursor_position: usize, lines: Vec<String>) -> Vec<String> {
    let mut new_lines: Vec<String> = vec![];

    for (i, line) in lines.iter().enumerate() {
        new_lines.push(format!(
            "{} {}",
            if cursor_position == i { ">" } else { " " },
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

fn move_cursor_up(line: usize) {
    if line != 0 {
        print!("\x1b[{}A", line);
    }
}

fn display(max_line_number: &mut usize, lines: Vec<String>) {
    if *max_line_number != 0 {
        move_cursor_up(*max_line_number + 1);
    }

    *max_line_number = lines.len();

    for line in lines {
        println!("{}", line);
    }
}

fn get_selected_files_path(files: &Vec<File>) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.is_selected)
        .map(|file| file.path.clone())
        .collect()
}

fn read_key(
    original_termios: &Termios,
    cursor_position: &mut usize,
    files: &mut Vec<File>,
) -> io::Result<()> {
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0; 3];

    stdout.lock().flush()?;
    reader.read(&mut buffer)?;

    if buffer[1] != 0 {
        return Ok(());
    }

    if let Some(key) = Keys::from_u8(buffer[0]) {
        match key {
            Keys::J => {
                if *cursor_position == files.len() - 1 {
                    *cursor_position = 0;
                } else {
                    *cursor_position = *cursor_position + 1;
                }
            }
            Keys::K => {
                if *cursor_position == 0 {
                    *cursor_position = files.len() - 1;
                } else {
                    *cursor_position = *cursor_position - 1;
                }
            }
            Keys::Q | Keys::Escape => {
                reset_terminal(original_termios);
                process::exit(0);
            }
            Keys::CtrlC => {
                reset_terminal(original_termios);
                process::exit(130);
            }
            Keys::Enter => {
                reset_terminal(original_termios);
                let paths = get_selected_files_path(&files);
                let output = git_add(paths);

                if output.status.success() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                    process::exit(0);
                } else {
                    print!("{}", String::from_utf8_lossy(&output.stderr));
                    match output.status.code() {
                        Some(code) => process::exit(code),
                        None => process::exit(1),
                    }
                }
            }
            Keys::Space => {
                files[*cursor_position].is_selected = !files[*cursor_position].is_selected
            }
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
    fn add_cursor_to_string() {
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];

        assert_eq!(
            add_cursor(1, lines),
            vec!["  Line 1", "> Line 2", "  Line 3"]
        );
    }

    #[test]
    fn joins_selected_files() {
        let files = vec![
            File {
                status: String::from("??"),
                path: String::from("/hello"),
                is_selected: true,
            },
            File {
                status: String::from(" M"),
                path: String::from("/it's-me"),
                is_selected: false,
            },
            File {
                status: String::from(" M"),
                path: String::from("/i-was-wondering"),
                is_selected: true,
            },
        ];

        assert_eq!(
            join_selected_files(&files),
            "/hello /i-was-wondering".to_string()
        )
    }
}
