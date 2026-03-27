use crate::analytics::{days_until_watering, get_avg_r, last_plant_feed};

use crate::analytics::days_from_last_feed;
use crate::db_operations;
use crate::models::Plant;
use crate::storage::save;

use chrono::NaiveDate;

pub fn last_feed(plants: &Vec<Plant>) -> String {
    let mut lines: Vec<String> = Vec::new();
    for plant in plants {
        let feed = match last_plant_feed(plant) {
            Some(value) => format!(
                "🌿 {} \n   ┗ последняя подкормка: {} ({} дн. назад) 🫧 ",
                plant.name,
                value,
                days_from_last_feed(plant)
            ),
            None => format!("🪴 {} \n   ┗ подкормок не было 🫙", plant.name),
        };
        lines.push(feed);
    }

    lines.join("\n")
}

//get average evapuation rate for each plant
pub async fn get_avr_r_for_each_plant(plant_id: i64) -> Result<u64> {
    let plant = db_operations::get_plant_measurements(pool, plant_id).await?;
    for plant in plants.iter_mut() {
        let avg = get_avg_r(plant);
        plant.update_avg_r(avg);
    }
}

// return days untill last watering
pub fn get_predicate(plants: &Vec<Plant>) -> String {
    let mut lines: Vec<String> = Vec::new();

    for plant in plants {
        let predicate = match days_until_watering(plant) {
            Some(value) => format!("🌱 {}: {}", plant.name, watering_status(value)),
            None => {
                format!("🌱 {}: нет данных", plant.name)
            }
        };
        lines.push(predicate);
    }

    lines.join("\n")
}
