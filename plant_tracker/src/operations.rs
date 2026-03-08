use crate::analytics::{days_until_watering, get_avg_r};

use crate::models::{Measurement, MeasurementType, Plant, watering_status};
use crate::storage::{load, save};

use chrono::NaiveDate;

pub fn get_avr_r_for_each_plant(plants: &mut [Plant]) {
    for plant in plants.iter_mut() {
        let avg = get_avg_r(plant);
        plant.update_avg_r(avg);
    }
}

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

fn find_plant<'a>(plants: &'a mut Vec<Plant>, choosen_id: u32) -> Option<&'a mut Plant> {
    plants.iter_mut().find(|p| p.id == choosen_id)
}
