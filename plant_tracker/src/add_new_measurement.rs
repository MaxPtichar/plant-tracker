use chrono::NaiveDate;
use teloxide::types;

use crate::models::{Measurement, MeasurementType, Plant, PlantType};

use crate::storage::{load, save};


pub fn add_new_measurement(plants:  &mut Vec<Plant>, id: u32, weight: f32, date: NaiveDate, type_: MeasurementType) {
    

    let plant = find_plant(plants, id).unwrap();

    let measure = Measurement {
        weight,
        date,
        type_,
    };

    plant.measurements.push(measure);
    save(&plants);
}



fn find_plant<'a>(plants: &'a mut Vec<Plant>, choosen_id: u32) -> Option<&'a mut Plant> {
    plants.iter_mut().find(|p| p.id == choosen_id)
}

