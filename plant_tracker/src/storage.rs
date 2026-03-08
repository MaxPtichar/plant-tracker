use std::fs;

use crate::models::Plant;
use chrono::Local;

pub fn save(plants: &[Plant]) {
    fs::create_dir_all("data").unwrap();
    let json = serde_json::to_string_pretty(plants).unwrap();
    let _ = fs::write("data/plants.json", json);
    backup(plants);
}

pub fn load() -> Vec<Plant> {
    match fs::read_to_string("data/plants.json") {
        Ok(content) => serde_json::from_str(&content).unwrap_or(vec![]),
        Err(_) => vec![],
    }
}

pub fn backup(plants: &[Plant]) {
    fs::create_dir_all("data/backups").unwrap();
    let filename = format!(
        "data/backups/plants_{}.json",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let json = serde_json::to_string_pretty(plants).unwrap();
    let _ = fs::write(&filename, json);

    clean_backups();
}

fn clean_backups() {
    let mut files: Vec<_> = fs::read_dir("data/backups")
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    files.sort_by_key(|e| e.file_name());

    if files.len() > 10 {
        fs::remove_file(files[0].path()).unwrap();
    }
}
