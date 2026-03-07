use chrono::NaiveDate;
use teloxide::types;

use crate::models::{Measurement, MeasurementType, Plant, PlantType};
use crate::build_app::get_avr_r_for_each_plant;

use crate::storage::{load, save};


pub fn add_new_measurement(plants:  &mut Vec<Plant>, id: u32, weight: f32, date: NaiveDate, type_: MeasurementType) {
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

