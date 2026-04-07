use sqlx::{PgPool, pool};

use crate::analytics::days_from_last_feed;
use crate::analytics_new::{avg_evaporation_rate, get_outdoor_temp, penman_monteith};
use crate::db_operations;
use crate::models::{Measurements, PlantFullContext, PlantWithLastFeedWatering};

/// Formats a list of plants with their last feed-watering date.
///
/// For each plant, shows the date and days elapsed since the last
/// measurement of type `AfterWateringWithFeed`.
///
/// # Cases
/// - Plant has feed record → shows date and days elapsed
/// - Plant has no feed record → shows "no feed yet" message
/// - Empty list → returns a placeholder message
///
/// # Example output
/// ```text
/// 🌿 Фикус
///    ┗ последняя подкормка: 2026-03-28 (3 дн. назад) 🫧
/// 🪴 Хлорофитум
///    ┗ подкормок не было 🫙
///
pub fn format_last_feed(plants: &[PlantWithLastFeedWatering]) -> String {
    if plants.is_empty() {
        return format!("🌱 Растений пока нет");
    }

    plants
        .iter()
        .map(|plant| match plant.date {
            Some(date) => format!(
                "🌿 {} \n   ┗ последняя подкормка: {} ({} дн. назад) 🫧 ",
                plant.plants_name,
                date,
                days_from_last_feed(plant).unwrap_or(0)
            ),
            None => format!("🪴 {} \n   ┗ подкормок не было 🫙", plant.plants_name,),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn transpiration_coef_calc(
    pool: &PgPool,
    plant_id: i64,
    user_id: i64,
) -> sqlx::Result<(f32, f32)> {
    let data = db_operations::get_all_plant_data(&pool, user_id, plant_id).await?;
    let last_measurements = db_operations::recieve_two_last_measurement(&pool, plant_id).await?;
    
    let PlantFullContext {
        plant_type,
        light_level,
        air_circulation,
        transpiration_coef,
        pot_diameter_cm,
        last_watering_weight,
        current_weight,
        avg_r,
        last_watering_date,
        ..
    } = &data;
if last_measurements.len() < 2 {
        return Ok((*transpiration_coef, avg_r.unwrap_or(0.0)));
    }
    let tem_c = get_outdoor_temp();

    let penman = penman_monteith(
        tem_c,
        plant_type,
        light_level,
        air_circulation,
        pot_diameter_cm.clone(),
        transpiration_coef.clone(),
    );
    let new_avg_r = match avg_evaporation_rate(&last_measurements) {
        Some(real_rate) => real_rate,
        None => match avg_r {
            &Some(hist_avg) if hist_avg > 0.0 => hist_avg,
            _ => transpiration_coef * penman,
        },
    };

    let (last_watered, current, last_watering_date) =
        match (last_watering_weight, current_weight, last_watering_date) {
            (Some(w_last), Some(w_curr), Some(d_last)) => (*w_last, *w_curr, *d_last),
            _ => return Ok((*transpiration_coef, new_avg_r)),
        };

    let duratiton_days = (chrono::Local::now().date_naive() - last_watering_date)
        .num_days()
        .abs() as f32;

    if duratiton_days <= 0.0 {
        return Ok((*transpiration_coef, avg_r.unwrap_or(0.0)));
    }

    let new_transpiration_coef = (last_watered - current) / (penman * duratiton_days);

    Ok((new_transpiration_coef, new_avg_r))
}

pub async fn update_avg_cycle(plant_id: i64, pool: &PgPool, user_id: i64) -> sqlx::Result<()> {
    let (new_avg_r, new_transpiration_coef) =
        transpiration_coef_calc(&pool, plant_id, user_id).await?;

    println!(
        "Обновление цикла для растения {}: New Kc: {:.2}, New Avg R: {:.2}",
        plant_id, new_transpiration_coef, new_avg_r
    );

    db_operations::update_plant_after_cycle(&pool, plant_id, new_avg_r, new_transpiration_coef)
        .await?;

    Ok(())
}
