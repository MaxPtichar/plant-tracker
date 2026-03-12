use crate::analytics::{days_until_watering, get_avg_r, last_plant_feed};

use crate::analytics::days_from_last_feed;
use crate::models::{Measurement, MeasurementType, Plant, watering_status};
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
pub fn get_avr_r_for_each_plant(plants: &mut [Plant]) {
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

// add new measurement
pub fn add_new_measurement(
    plants: &mut Vec<Plant>,
    id: u32,
    weight: f32,
    date: NaiveDate,
    type_: MeasurementType,
) {
    let measure = Measurement {
        weight,
        date,
        type_,
    };

    {
        let plant = find_plant(plants, id).unwrap();

        plant.measurements.push(measure);
    }
    get_avr_r_for_each_plant(plants);
    save(&plants);
}

//find plant in vec<Plants> from load() func
fn find_plant<'a>(plants: &'a mut Vec<Plant>, choosen_id: u32) -> Option<&'a mut Plant> {
    plants.iter_mut().find(|p| p.id == choosen_id)
}
