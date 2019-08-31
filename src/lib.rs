use std::env;
use std::process;

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

pub fn git_add(paths: Vec<String>) -> process::Output {
    process::Command::new("git")
        .arg("add")
        .args(paths)
        .output()
        .expect("Failed to add files")
}

pub fn marshal_statuses_into_paths(statuses: Vec<String>) -> Result<Vec<String>, &'static str> {
    return statuses
        .iter()
        .map(|status| -> Result<String, &str> {
            let mut path = status[3..].to_string();

            if path.contains("->") {
                path = match path.split_whitespace().nth(2) {
                    None => return Err("Failed to parse status"),
                    Some(p) => p.to_string(),
                }
            }

            let path_from_git_root = format!(":/{}", path);

            Ok(path_from_git_root)
        })
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_statuses_into_paths() {
        let statuses = vec![
            " M src/main.rs".to_string(),
            "?? wow".to_string(),
            "CM src/wow.rs -> src/lib.rs".to_string(),
        ];

        let results = marshal_statuses_into_paths(statuses);

        assert_eq!(
            results,
            Ok(vec![
                ":/src/main.rs".to_string(),
                ":/wow".to_string(),
                ":/src/lib.rs".to_string(),
            ])
        )
    }

    #[test]
    fn returns_error_if_status_is_malformed() {
        let statuses = vec![
            " M src/main.rs".to_string(),
            "?? wow".to_string(),
            "CM src/wow.rs ->".to_string(),
        ];

        let error = marshal_statuses_into_paths(statuses);

        assert_eq!(error, Err("Failed to parse status"))
    }
}
