use std::fs;

use crate::models::Plant;

pub fn save(plants: &[Plant]) {
    fs::create_dir_all("data").unwrap();
    let json = serde_json::to_string_pretty(plants).unwrap();
    let _ = fs::write("data/plants.json", json);
    backup(plants);
}

pub fn load() -> Vec<Plant> {
    let path = "data/plants.json";

    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => {
            
            if let Ok(initial_json) = std::env::var("INITIAL_PLANTS_JSON") {
                let _ = fs::create_dir_all("data");
                let _ = fs::write(path, &initial_json);
                println!("🚀 База данных инициализирована из INITIAL_PLANTS_JSON");
                return serde_json::from_str(&initial_json).unwrap_or_default();
            }
            
            
            vec![]
        }
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
