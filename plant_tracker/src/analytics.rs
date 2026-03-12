use chrono::{Days, NaiveDate};
use serde::de::value;

use crate::models::{Measurement, MeasurementType, Plant, PlantBehavoir};

//return last date when was watering with feed
pub fn last_plant_feed(plant: &Plant) -> Option<NaiveDate> {
    plant
        .measurements
        .iter()
        .filter_map(|f| {
            if f.type_ == MeasurementType::AfterWateringWithFeed {
                Some(f.date)
            } else {
                None
            }
        })
        .last()
}

pub fn days_from_last_feed(plant: &Plant) -> u32 {
    let date = match last_plant_feed(plant) {
        Some(value) => value,
        _ => return 0,
    };

    (chrono::Local::now().date_naive() - date).num_days() as u32
}

/// return days for watering
pub fn days_until_watering(plant: &Plant) -> Option<f32> {
    let m_current = match get_last_measure(plant) {
        Some(value) => value,
        None => return None,
    };

    let target_weight = get_target_weight(plant);

    let r = get_evaporation_rate(plant);

    if r == 0.0 {
        return None;
    }

    let today = chrono::Local::now().date_naive();

    let last_measurements = plant.measurements.last()?;
    let days_since = (today - last_measurements.date).num_days() as f32;

    let days_left = (m_current - target_weight) / r - days_since;

    Some(days_left)
}

/// evaputaion rate need for predicate days untill watering
pub fn get_evaporation_rate(plant: &Plant) -> f32 {
    plant
        .avg_r
        .unwrap_or(plant.plant_type.avg_water_loss_per_day())
}

/// need for days untill watering
fn get_last_measure(plant: &Plant) -> Option<f32> {
    plant.measurements.iter().map(|w| w.weight).last()
}

///return vector which conntain three dates for delta time calculation

/// need for calculation predicate
fn get_target_weight(plant: &Plant) -> f32 {
    let dry_weight = get_dry_weight(plant);

    let water_threshold = get_watering_threshold(plant);

    let m_after = match get_after_watering(plant) {
        Some(value) => value,
        None => {
            dry_weight
                * plant.plant_type.avg_water_loss_per_day()
                * plant.plant_type.avg_days_beetwen_watering()
                * water_threshold
        }
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
    plant
        .measurements
        .iter()
        .filter(|m| {
            m.type_ == MeasurementType::AfterWatering
                || m.type_ == MeasurementType::AfterWateringWithFeed
        })
        .last()
        .map(|w| w.weight)
}

///temper funcs for add to json new filed avg_r
///

pub fn get_avg_r(plant: &Plant) -> Option<f32> {
    let indx = find_indx_after_watering(plant);

    let slice = &get_slice(indx);
    let mut all_r = calculate_r_per_cycle(plant, slice);

    if let Some(r_current) = get_current_cycle_r(plant) {
        all_r.push(r_current);
    }

    if all_r.is_empty() {
        return None;
    }

    let avg = all_r.iter().sum::<f32>() / all_r.len() as f32;

    Some(avg)
}

fn find_indx_after_watering(plant: &Plant) -> Vec<usize> {
    plant
        .measurements
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.type_ == MeasurementType::AfterWatering
                || m.type_ == MeasurementType::AfterWateringWithFeed
        })
        .map(|(i, _)| i)
        .collect()
}

fn get_slice<'a>(indx: Vec<usize>) -> Vec<[usize; 2]> {
    let mut indexcies: Vec<[usize; 2]> = vec![];

    let mut s: usize = 0;
    let mut f: usize = 1;

    while f < indx.len() {
        indexcies.push([indx[s], indx[f]]);
        s += 1;
        f += 1;
    }

    indexcies
}

fn calculate_r_per_cycle(plant: &Plant, slice: &[[usize; 2]]) -> Vec<f32> {
    slice
        .iter()
        .flat_map(|[start, end]| {
            let cycle = &plant.measurements[*start..=*end];

            let regulars: Vec<&Measurement> = cycle
                .iter()
                .filter(|x| x.type_ == MeasurementType::Regular)
                .collect();

            if regulars.len() < 2 {
                return None;
            }

            let firts = regulars.first().unwrap();
            let last = regulars.last().unwrap();

            let delta_m = firts.weight - last.weight;
            let delta_t = (last.date - firts.date).num_days() as f32;

            if delta_t == 0.0 {
                return None;
            }

            Some(delta_m / delta_t)
        })
        .collect()
}

fn get_current_cycle_r(plant: &Plant) -> Option<f32> {
    let last_watering_idx = plant.measurements.iter().rposition(|t| {
        t.type_ == MeasurementType::AfterWatering
            || t.type_ == MeasurementType::AfterWateringWithFeed
    })?;

    let regulars: Vec<&Measurement> = plant.measurements[last_watering_idx + 1..]
        .iter()
        .filter(|m| m.type_ == MeasurementType::Regular)
        .collect();

    if regulars.len() < 2 {
        return None;
    }

    let firts = regulars.first().unwrap();
    let last = regulars.last().unwrap();

    let delta_m = firts.weight - last.weight;
    let delta_t = (last.date - firts.date).num_days() as f32;

    if delta_t == 0.0 {
        return None;
    }

    Some(delta_m / delta_t)
}
