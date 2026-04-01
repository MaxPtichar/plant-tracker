use chrono::NaiveDate;
use crate::{db_operations, models::{Measurements, PlantWithLastFeedWatering, PotConfig}};

// ============================================================
// Plant watering analytics
//
// Calculates days until next watering based on weight measurements.
//
// Calculation pipeline:
//   avg_evaporation_rate  → rate of water loss per day (g/day)
//   target_weight         → weight at which watering is needed
//   days_left             → days remaining until target is reached
//   days_until_watering   → orchestrates all three (entry point)
// ============================================================

/// Calculates the average evaporation rate from two consecutive
/// `Regular` measurements, ordered newest first.
///
/// # Returns
/// - `Some(rate)` — grams lost per day
/// - `None` — fewer than 2 measurements provided
///
/// # Example
/// ```
/// // weight1=500g on 2026-03-10, weight2=450g on 2026-03-05
/// // rate = (500 - 450) / 5 = 10.0 g/day
/// ```
pub fn avg_evaporation_rate(last_measurements: &[Measurements]) -> Option<f32> { 
    if last_measurements.len() < 2 {
        return None;
    }
    
    let (weight1, weight2) = (last_measurements[0].weight, last_measurements[1].weight);
    let (date1, date2) = (last_measurements[0].date, last_measurements[1].date);

    let delta_days = (date1 -date2 ).abs().num_days().max(1) as f32;
    let evapuation_rate = ( weight2  - weight1).abs() / delta_days;
   
    Some(evapuation_rate as f32)

}

/// Calculates the target weight at which the plant needs watering.
///
/// Formula: `(pot_weight + dry_soil_weight) + (after_watering - dry_total) * target_moisture`
///
/// # Parameters
/// - `pot` — active pot config from [`db_operations::get_active_config`]
/// - `target_moisture` — moisture fraction `0.0..=1.0` from [`Plant`]
/// - `after_watering` — weight recorded after last watering ([`Measurements`])
fn target_weight(pot: &PotConfig, target_moisture: f32, after_watering: f32 ) -> f32 {
     let dry_total = pot.pot_weight as f32 + pot.dry_soil_weight as f32;
    dry_total + (after_watering - dry_total) * target_moisture
}

/// Calculates days remaining until the plant needs watering.
///
/// Formula: `(curr_weight - target) / r - days_since_last_measurement`
///
/// Negative result means watering is overdue.
fn days_left(curr_weight: f32, r: f32, days_from_last_weight: i64, target: f32) -> f32 {
    (curr_weight - target) / r - days_from_last_weight as f32
}
/// Entry point — calculates days until next watering for a plant.
///
/// Requires:
/// - `measurements` — last 2 `Regular` measurements, newest first
/// - `pot` — active [`PotConfig`] for the plant
/// - `target_moisture` — from [`Plant::target_moisture`]
/// - `after_watering_weight` — weight from last `AfterWatering` measurement
///
/// # Returns
/// - `Some(days)` — days until watering (negative = overdue)
/// - `None` — not enough data to calculate
pub fn days_until_watering(
    measurements: &[Measurements],
    pot: &PotConfig,
    target_moisture: f32,
    after_watering_weight: f32,
) -> Option<f32> {
    let r = avg_evaporation_rate(measurements)?;
    let curr_weight = measurements[0].weight;
    let last_date = measurements[0].date;
    let days_since = (chrono::Local::now().date_naive() - last_date).num_days();
    let target = target_weight(&pot, target_moisture, after_watering_weight);

    Some(days_left(curr_weight, r, days_since, target))
}

/// Returns days elapsed since the last feed-watering measurement.
///
/// # Returns
/// - `Some(days)` — days since last `AfterWateringWithFeed`
/// - `None` — no feed-watering recorded yet
pub fn days_from_last_feed(last_feed: &PlantWithLastFeedWatering) -> Option<u32> {
    let date = last_feed.date?;
    let days = (chrono::Local::now().date_naive() - date).num_days();

    Some(days.max(0) as u32)
}









#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_measurement(weight: f32, date: &str) -> Measurements {
        Measurements {
            id: 1,
            plant_id: 1,
            weight,
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            measuring_type: "Regular".to_string(),
        }
    }

    fn make_pot(pot_weight: i64, dry_soil_weight: i64) -> PotConfig {
        PotConfig {
            id: 1,
            plant_id: 1,
            pot_weight,
            dry_soil_weight,
            is_active: true,
        }
    }

    // --- avg_evaporation_rate ---

    #[test]
    fn test_evaporation_rate_normal() {
        let m = vec![
            make_measurement(500.0, "2026-03-10"),
            make_measurement(450.0, "2026-03-05"),
        ];
        assert_eq!(avg_evaporation_rate(&m), Some(10.0));
    }

    #[test]
    fn test_evaporation_rate_too_few() {
        let m = vec![make_measurement(500.0, "2026-03-10")];
        assert_eq!(avg_evaporation_rate(&m), None);
    }

    #[test]
    fn test_evaporation_rate_empty() {
        assert_eq!(avg_evaporation_rate(&[]), None);
    }

    #[test]
    fn test_evaporation_rate_same_date() {
        let m = vec![
            make_measurement(500.0, "2026-03-10"),
            make_measurement(450.0, "2026-03-10"),
        ];
        // delta_days = 0 → max(1) = 1
        assert_eq!(avg_evaporation_rate(&m), Some(50.0));
    }

    // --- target_weight ---

    #[test]
    fn test_target_weight_normal() {
        let pot = make_pot(200, 300); // dry_total = 500
        // target = 500 + (800 - 500) * 0.3 = 500 + 90 = 590
        assert_eq!(target_weight(&pot, 0.3, 800.0), 590.0);
    }

    #[test]
    fn test_target_weight_zero_moisture() {
        let pot = make_pot(200, 300); // dry_total = 500
        // target = 500 + (800 - 500) * 0.0 = 500
        assert_eq!(target_weight(&pot, 0.0, 800.0), 500.0);
    }

    // --- days_until_watering ---

    #[test]
    fn test_days_until_watering_not_enough_data() {
        let m = vec![make_measurement(500.0, "2026-03-10")];
        let pot = make_pot(200, 300);
        assert_eq!(days_until_watering(&m, &pot, 0.3, 800.0), None);
    }
}








// pub fn days_until_watering(measurement: &[Measurements]) -> Option<f32> {


//     let (prew_weight, current_weight) = measurement.iter().map(|plant| pr)


//     let m_current: f32 = measurement.weight;

//     let target_weight = get_target_weight(plant);
//     /// stop here
//     let r = get_evaporation_rate(plant);

//     if r == 0.0 {
//         return None;
//     }

//     let today = chrono::Local::now().date_naive();

//     let last_measurements = plant.measurements.last()?;
//     let days_since = (today - last_measurements.date).num_days() as f32;

//     let days_left = (m_current - target_weight) / r - days_since;

//     Some(days_left)
// }


// /// need for calculation predicate
// fn get_target_weight(plant: &Plant) -> f32 {
//     let dry_weight = get_dry_weight(plant);

//     let water_threshold = get_watering_threshold(plant);

//     let m_after = match get_after_watering(plant) {
//         Some(value) => value,
//         None => {
//             dry_weight
//                 * plant.plant_type.avg_water_loss_per_day()
//                 * plant.plant_type.avg_days_beetwen_watering()
//                 * water_threshold
//         }
//     };

//     let target_weight = dry_weight + (m_after - dry_weight) * water_threshold;

//     target_weight
// }


// ///---------------------

// //return last date when was watering with feed
// pub fn last_plant_feed(plant_last_feed: &PlantWithLastFeedWatering) -> Option<NaiveDate> {
//     plant
//         .measurements
//         .iter()
//         .filter_map(|f| {
//             if f.type_ == MeasurementType::AfterWateringWithFeed {
//                 Some(f.date)
//             } else {
//                 None
//             }
//         })
//         .last()
// }


////--------------

// /// evaputaion rate need for predicate days untill watering
// pub fn get_evaporation_rate(plant: &Plant) -> f32 {
//     plant
//         .avg_r
//         .unwrap_or(plant.plant_type.avg_water_loss_per_day())
// }

// /// need for days untill watering
// fn get_last_measure(plant: &Plant) -> Option<f32> {
//     plant.measurements.iter().map(|w| w.weight).last()
// }

// ///return vector which conntain three dates for delta time calculation



// /// return dry weight for calculation targer weight
// fn get_dry_weight(plant: &Plant) -> f32 {
//     plant.plant_type.dry_weight()
// }

// /// return watering thresold for calculation targer weight
// fn get_watering_threshold(plant: &Plant) -> f32 {
//     plant.plant_type.watering_threshold()
// }

// /// Return weight who has a MeasurementType (AfterWatering or AfterWateringWithFeed) for calculation target weight - it is wieght when watering needed
// fn get_after_watering(plant: &Plant) -> Option<f32> {
//     plant
//         .measurements
//         .iter()
//         .filter(|m| {
//             m.type_ == MeasurementType::AfterWatering
//                 || m.type_ == MeasurementType::AfterWateringWithFeed
//         })
//         .last()
//         .map(|w| w.weight)
// }

// ///temper funcs for add to json new filed avg_r
// ///

// pub fn get_avg_r(plant: &Plant) -> Option<f32> {
//     let indx = find_indx_after_watering(plant);

//     let slice = &get_slice(indx);
//     let mut all_r = calculate_r_per_cycle(plant, slice);

//     if let Some(r_current) = get_current_cycle_r(plant) {
//         all_r.push(r_current);
//     }

//     if all_r.is_empty() {
//         return None;
//     }

//     let avg = all_r.iter().sum::<f32>() / all_r.len() as f32;

//     Some(avg)
// }

// fn find_indx_after_watering(plant: &Plant) -> Vec<usize> {
//     plant
//         .measurements
//         .iter()
//         .enumerate()
//         .filter(|(_, m)| {
//             m.type_ == MeasurementType::AfterWatering
//                 || m.type_ == MeasurementType::AfterWateringWithFeed
//         })
//         .map(|(i, _)| i)
//         .collect()
// }

// fn get_slice<'a>(indx: Vec<usize>) -> Vec<[usize; 2]> {
//     let mut indexcies: Vec<[usize; 2]> = vec![];

//     let mut s: usize = 0;
//     let mut f: usize = 1;

//     while f < indx.len() {
//         indexcies.push([indx[s], indx[f]]);
//         s += 1;
//         f += 1;
//     }

//     indexcies
// }

// fn calculate_r_per_cycle(plant: &Plant, slice: &[[usize; 2]]) -> Vec<f32> {
//     slice
//         .iter()
//         .flat_map(|[start, end]| {
//             let cycle = &plant.measurements[*start..=*end];

//             let regulars: Vec<&Measurement> = cycle
//                 .iter()
//                 .filter(|x| x.type_ == MeasurementType::Regular)
//                 .collect();

//             if regulars.len() < 2 {
//                 return None;
//             }

//             let firts = regulars.first().unwrap();
//             let last = regulars.last().unwrap();

//             let delta_m = firts.weight - last.weight;
//             let delta_t = (last.date - firts.date).num_days() as f32;

//             if delta_t == 0.0 {
//                 return None;
//             }

//             Some(delta_m / delta_t)
//         })
//         .collect()
// }

// fn get_current_cycle_r(plant: &Plant) -> Option<f32> {
//     let last_watering_idx = plant.measurements.iter().rposition(|t| {
//         t.type_ == MeasurementType::AfterWatering
//             || t.type_ == MeasurementType::AfterWateringWithFeed
//     })?;

//     let regulars: Vec<&Measurement> = plant.measurements[last_watering_idx + 1..]
//         .iter()
//         .filter(|m| m.type_ == MeasurementType::Regular)
//         .collect();

//     if regulars.len() < 2 {
//         return None;
//     }

//     let firts = regulars.first().unwrap();
//     let last = regulars.last().unwrap();

//     let delta_m = firts.weight - last.weight;
//     let delta_t = (last.date - firts.date).num_days() as f32;

//     if delta_t == 0.0 {
//         return None;
//     }

//     Some(delta_m / delta_t)
// }
