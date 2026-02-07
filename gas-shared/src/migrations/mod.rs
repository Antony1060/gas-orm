use crate::error::GasSharedError;
use std::path::PathBuf;

const SCRIPT_SEPARATOR: &str = "-- GAS_ORM(forward_backward_separator)";

// TODO: safety
pub fn parse_migrations_from_dir(
    project_root: &str,
    dir: &str,
) -> Result<Vec<(String, String)>, GasSharedError> {
    let scripts_path = PathBuf::from(project_root).join(dir).join("scripts");

    if !scripts_path.exists() || !scripts_path.is_dir() {
        return Err(GasSharedError::MigrationsNotDefined);
    }

    let files: Vec<_> = {
        let mut files: Vec<_> = std::fs::read_dir(scripts_path)
            .expect("read_dir failed")
            // error safety is my passion
            .map(Result::unwrap)
            .filter(|file| file.file_type().unwrap().is_file())
            .map(|file| file.path().display().to_string())
            .filter(|path| path.ends_with(".sql"))
            .collect();

        files.sort();

        files
    };

    let script_contents_raw: Vec<_> = files
        .into_iter()
        .map(|path| std::fs::read_to_string(path).expect("failed to read file"))
        .collect();

    let mut parsed_scripts: Vec<(String, String)> = Vec::with_capacity(script_contents_raw.len());
    for script in script_contents_raw.iter() {
        let (forward, backward) = script
            .split_once(SCRIPT_SEPARATOR)
            .expect("failed to parse migration script");

        parsed_scripts.push((forward.to_string(), backward.to_string()));
    }

    Ok(parsed_scripts)
}
