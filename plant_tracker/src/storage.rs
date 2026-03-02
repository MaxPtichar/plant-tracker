use std::fs;

use crate::models::Plant;

pub fn save(plants: &[Plant]) {
    fs::create_dir_all("data").unwrap();
    let json = serde_json::to_string_pretty(plants).unwrap();
    let _ = fs::write("data/plants.json", json);
}

pub fn load() -> Vec<Plant> {
    match fs::read_to_string("data/plants.json") {
        Ok(content) => serde_json::from_str(&content).unwrap_or(vec![]),
        Err(_) => vec![],
    }
}
