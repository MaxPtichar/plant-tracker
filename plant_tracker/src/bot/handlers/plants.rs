use sqlx::PgPool;

use crate::{
    analytics_new::{days_until_watering_full, get_outdoor_temp},
    db_operations,
    models::{PlantFullContext, watering_status},
};

/// Returns a formatted watering status string for all user's plants.
///
/// For each plant, calculates days until next watering based on:
/// - last 2 `Regular` measurements (evaporation rate)
/// - active pot configuration
/// - last `AfterWatering` measurement weight
///
/// # Returns
/// - One line per plant: `"🌱 {name} — {status}"`
/// - `"нет данных"` if fewer than 2 measurements available
/// - `"не настроено"` if no active pot config or no watering recorded
/// - `"Пока еще нет ни одного растения🌱"` if user has no plants
pub async fn get_all_plants_status(pool: &PgPool, chat_id: i64) -> sqlx::Result<String> {
    let plants = db_operations::get_all_plants_data(pool, chat_id).await?;
    if plants.is_empty() {
        return Ok("Пока еще нет ни одного растения🌱".to_string());
    }

    let mut result: Vec<String> = Vec::new();
    let temp_outdoor = get_outdoor_temp();

    for plant in &plants {
        let PlantFullContext {
            current_weight: _,
            last_watering_weight,
            last_watering_date,
            pot_weight,
            dry_soil_weight,
            soil_type,
            plant_type,
            light_level,
            air_circulation,
            pot_diameter_cm,
            transpiration_coef,
            avg_r,
            cycles_count,
            plants_name,
            plant_id,
            water_threshold,
            ..
        } = &plant;
        dbg!(plants_name, last_watering_weight, avg_r, water_threshold);

        let dry_total = (pot_weight + dry_soil_weight) as f32;
        let dry_soil_weight_g = *dry_soil_weight as f32;

        let regular_measurements =
            db_operations::get_regular_after_last_watering(pool, *plant_id).await?;
        dbg!(&regular_measurements);

        dbg!((last_watering_weight, avg_r));

        let (after_watering_weight, avg_r) = match (last_watering_weight, avg_r) {
            (&Some(l_watering), &Some(r)) => (l_watering, r),
            _ => {
                result.push(format!("{plants_name} - недостаточно данных для расчета").to_string());
                continue;
            }
        };

        let days = days_until_watering_full(
            &regular_measurements,
            *last_watering_date,
            after_watering_weight,
            dry_total,
            soil_type,
            dry_soil_weight_g,
            plant_type,
            light_level,
            air_circulation,
            *pot_diameter_cm,
            *transpiration_coef,
            avg_r,
            *cycles_count,
            temp_outdoor,
            *water_threshold,
        );

        dbg!(&days);

        let line = match days {
            Some(d) => format!("🌱 {} — {}", plants_name, watering_status(d)),
            _ => format!("🌱 {} — нет данных", plants_name),
        };

        result.push(line);
    }

    Ok(result.join("\n"))
}

// pub async fn get_all_plants_status_old(pool: &PgPool, chat_id: i64) -> sqlx::Result<String> {
//     let plants = db_operations::get_user_plants(&pool, chat_id).await?;
//     if plants.is_empty() {
//         return Ok(format!("Пока еще нет ни одного растения🌱"));
//     }
//     let mut result: Vec<String> = Vec::new();

//     for plant in &plants {
//         let measurements = db_operations::recieve_two_last_measurement(&pool, plant.id).await?;
//         let pot = db_operations::get_active_config(&pool, plant.id).await?;
//         let target_moisture = 2.0;
//         let after_watering_weight =
//             db_operations::get_last_watering_weight(&pool, plant.id).await?;
//         dbg!(&pot);
//         dbg!(&after_watering_weight);

//         let line = match (pot, after_watering_weight) {
//             (Some(pot), Some(after_watering_weight)) => {
//                 match days_until_watering(
//                     &measurements,
//                     &pot,
//                     target_moisture,
//                     after_watering_weight,
//                 ) {
//                     Some(days) => format!("🌱 {} — {}", plant.plants_name, watering_status(days)),
//                     None => format!("🌱 {} — нет данных", plant.plants_name),
//                 }
//             }

//             _ => format!("🌱 {} — не настроено", plant.plants_name),
//         };

//         result.push(line);
//     }

//     Ok(result.join("\n"))
// }

/// Returns a formatted list of all user's plants with their configuration details.
///
/// Each entry includes:
/// - plant name
/// - target moisture percentage
/// - pot weight and dry soil weight
/// - date of last measurement (any type), or `"Нет данных"` if none
///
/// # Returns
/// - Entries separated by `"─────────────"`
/// - `"Пока еще нет ни одного растения🌱"` if user has no plants
pub async fn get_list_of_all_plants(pool: &PgPool, chat_id: i64) -> sqlx::Result<String> {
    let plants = db_operations::get_list_of_all_user_plants(pool, chat_id).await?;
    if plants.is_empty() {
        return Ok("Пока еще нет ни одного растения🌱".to_string());
    }

    let result = plants
        .iter()
        .map(|plant| {
            format!(
                "🌱 *{}*\n\
         💧 Целевая влажность: {}%\n\
         🪴 Масса горшка: {} г | Сухая земля: {} г\n\
         📊 Последнее измерение: {}\n\
        ",
                plant.plants_name,
                (0.3 * 100.0),
                plant.pot_weight,
                plant.dry_soil_weight,
                plant
                    .last_measurement_date
                    .map(|date| date.to_string())
                    .unwrap_or_else(|| "Нет данных".to_string()),
            )
        })
        .collect::<Vec<_>>()
        .join("\n─────────────\n");

    Ok(result)
}
