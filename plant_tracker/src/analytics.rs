use crate::{ models::{MeasurementType, Plant, PlantBehavoir}};
use chrono::{TimeDelta, NaiveDate};


/// return days for watering
pub fn days_until_watering(plant: &Plant) -> Option<f32> {
    
    let m_current = match get_last_measure(plant) {
        Some(value) => value, 
        None => return None,
        
    };

    let target_weight = get_target_weight(plant);

    let r = get_evaporation_rate(plant);



    let days_left = (m_current  - target_weight ) / r;

    Some(days_left)

}


/// evaputaion rate need for predicate days untill watering 
fn get_evaporation_rate(plant: &Plant) -> f32 {
    let weights = get_three_last_measure(plant);

    if weights.len() < 3 { 
        return plant.plant_type.avg_water_loss_per_day();
    }
    let m_last = weights[0];
    let m_first = weights[2];

    let days_beetween = days_between_last_three(plant);


    
    let evapuation_rate = (m_first - m_last) / days_beetween;

    evapuation_rate




}



/// need for evapuation rate
fn get_three_last_measure (plant: &Plant) -> Vec<f32> { 
     plant.measurements
    .iter()
    .filter(|m| m.type_ == MeasurementType::Regular)
    .rev()
    .take(3)

    .map(|w| w.weight)
    .collect()
    
}


/// need for days untill watering 
fn get_last_measure (plant: &Plant) -> Option<f32> { 
     plant.measurements
    .iter()
    .map(|w| w.weight)
    .last()
    
}



///return vector which conntain three dates for delta time calculation
fn get_three_last_date(plant: &Plant) -> Vec<NaiveDate> { 
     plant.measurements
    .iter()
    
    .rev()
    .take(3)

    .map(|w| w.date)
    .collect()
    
}



/// return delta time beetween first and last date(which is 3) for calculating evapuration rate
fn days_between_last_three (plant:&Plant) -> f32 {
    let dates = get_three_last_date(plant);

    if dates.len() < 3 {
        return  plant.plant_type.avg_days_beetwen_watering();
    }
    
    let date1:NaiveDate = dates[2]; // days beetween last two
    let date2: NaiveDate = dates[0];
    let delta_t: TimeDelta = date2 - date1;
    delta_t.num_days() as f32 
}



/// need for calculation predicate 
fn get_target_weight(plant: &Plant) -> f32 {
    let dry_weight = get_dry_weight(plant);

    let water_threshold = get_watering_threshold(plant);

    
    let m_after = match get_after_watering(plant) {
        Some(value) => value, 
        None => dry_weight * plant.plant_type.avg_water_loss_per_day() * plant.plant_type.avg_days_beetwen_watering() * water_threshold,
    };

    let target_weight = dry_weight + (m_after - dry_weight) * water_threshold;
    

    target_weight

    

}


/// return dry weight for calculation targer weight
fn get_dry_weight(plant: &Plant) -> f32 {
    plant.plant_type.dry_weight()

}

/// return watering thresold for calculation targer weight
fn get_watering_threshold(plant: &Plant) -> f32 {
    plant.plant_type.watering_threshold()

}


/// Return weight who has a MeasurementType (AfterWatering or AfterWateringWithFeed) for calculation target weight - it is wieght when watering needed 
fn get_after_watering(plant: &Plant) -> Option<f32> {
    plant.measurements.iter()
    .filter(|m| m.type_== MeasurementType::AfterWatering
    || m.type_ == MeasurementType::AfterWateringWithFeed)
    .last()
    .map(|w| w.weight)
}


///temper funcs for add to json new filed avg_r