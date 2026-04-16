use sqlx::{PgPool, pool};

use crate::analytics::days_from_last_feed;
use crate::analytics_new::{avg_evaporation_rate, get_outdoor_temp, penman_monteith, raw};
use crate::db_operations::{self, get_geo_data};
use crate::models::{Measurements, PlantFullContext, PlantWithLastFeedWatering, UserGeo, WeatherResponse};

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
    let last_measurements = db_operations::get_regular_after_last_watering(&pool, plant_id).await?;

    let PlantFullContext {
        plant_type,
        light_level,
        air_circulation,
        transpiration_coef: old_coef,
        pot_diameter_cm,
        last_watering_weight,
        soil_type,
        dry_soil_weight,
        avg_r,
        last_watering_date,
        pot_weight, // Убедись, что это поле есть в структуре
        ..
    } = &data;

    // Если мало данных — не гадаем, возвращаем старое
    if last_measurements.len() < 2 {
        return Ok((*old_coef, avg_r.unwrap_or(0.1)));
    }

    let last_two: Vec<_> = last_measurements.iter().take(2).cloned().collect();
    let tem_c = get_outdoor_temp();

    dbg!(&tem_c);

    // 1. Базовый расход по Пенману-Монтейту
    let penman = penman_monteith(
        tem_c,
        plant_type,
        light_level,
        air_circulation,
        pot_diameter_cm.clone(),
        1.0,
    );

    let new_avg_r = match avg_evaporation_rate(&last_two) {
        Some(real_rate) => real_rate,
        None => avg_r.unwrap_or(old_coef * penman),
    };

    if penman <= 0.0001 {
        return Ok((*old_coef, new_avg_r));
    }

    // 2. Расчет временного интервала
    let (last_watered, d_last) = match (last_watering_weight, last_watering_date) {
        (Some(w), Some(d)) => (*w, *d),
        _ => return Ok((*old_coef, new_avg_r)),
    };

    let duration_days = (chrono::Local::now().date_naive() - d_last).num_days() as f32;
    
    // Защита: не пересчитываем коэффициент слишком часто (нужно хотя бы 2-3 дня данных)
    if duration_days < 2.0 {
        return Ok((*old_coef, new_avg_r));
    }

    // 3. Расчет фактической потери и Ks (стресс-фактор)
    let current_weight = last_measurements.first().unwrap().weight;
    let water_loss = last_watered - current_weight;

    if water_loss <= 0.0 {
        return Ok((*old_coef, new_avg_r));
    }

    let dry_soil_weight_g = *dry_soil_weight as f32;
    let pot_w = *pot_weight as f32;
    let (raw_val, _) = raw(soil_type, dry_soil_weight_g, plant_type);

    // Ks должен считаться от остатка доступной воды, а не от общего веса
    // Доступная вода сейчас = (Текущий вес - Вес горшка - Вес сухого грунта)
    let current_available_water = (current_weight - pot_w - dry_soil_weight_g).max(0.0);
    let ks = (current_available_water / raw_val).clamp(0.1, 1.0);

    // 4. Вычисление сырого нового коэффициента
    // actual_daily_loss = water_loss / duration
    // expected_loss = penman * ks
    let raw_new_coef = (water_loss / duration_days) / (penman * ks);

    // 5. EMA (Экспоненциальное сглаживание) + Clamp
    // Это не даст системе "паниковать" из-за одного неверного взвешивания
    let alpha = 0.25; // Степень доверия новому замеру (25%)
    let smoothed_coef = (raw_new_coef * alpha) + (old_coef * (1.0 - alpha));
    
    // Ограничиваем разумными пределами для домашних условий
    let final_coef = smoothed_coef.clamp(0.2, 2.0);

    println!(
        "ID {}: Loss: {:.1}g, Days: {:.1}, Ks: {:.2}, Raw_Kc: {:.2}, Final_Kc: {:.2}",
        plant_id, water_loss, duration_days, ks, raw_new_coef, final_coef
    );

    Ok((final_coef, new_avg_r))
}

pub async fn update_avg_cycle(plant_id: i64, pool: &PgPool, user_id: i64) -> sqlx::Result<()> {

    let (new_transpiration_coef, new_avg_r) =
    transpiration_coef_calc(&pool, plant_id, user_id).await?;

    println!(
        "Обновление цикла для растения {}: New Kc: {:.2}, New Avg R: {:.2}",
        plant_id, new_transpiration_coef, new_avg_r
    );

    db_operations::update_plant_after_cycle(&pool, plant_id, new_avg_r, new_transpiration_coef)
        .await?;

    Ok(())
}







