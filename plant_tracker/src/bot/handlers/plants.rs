use std::ops::Mul;

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    analytycs_v2::{daily_water_loss, days_until_watering, watering_point},
    db_operations::{self, get_last_measurement},
    models::{PlantMeasurementsHistory, watering_status},
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
    let plants = db_operations::get_list_of_all_user_plants(pool, chat_id).await?;
    if plants.is_empty() {
        return Ok("Пока еще нет ни одного растения🌱".to_string());
    }

    let mut result: Vec<String> = Vec::new();
    let mut show_calibration_hint = false;

    for plant in plants {
        let dry_weight = plant.dry_weight.unwrap();
        let wet_weight = plant.wet_weight.unwrap();
        let threshold_pct = plant.threshold_pct.unwrap();
        let learned_daily_loss = plant.learned_daily_loss;

        let measurements =
            db_operations::get_measurement_record_20(pool, plant.id, chat_id).await?;

        let fm = format!("🪴 {} │ нет данных", plant.plants_name);
        if measurements.is_empty() {
            result.push(fm);
            continue;
        }

        let watering_point = watering_point(dry_weight, wet_weight, threshold_pct);
        let Some((current_weight, last_m_date)) = get_last_measurement(pool, plant.id).await?
        else {
            result.push(fm);
            continue;
        };

        if let Some(ldl) = learned_daily_loss {
            let fm = format_plant_status(
                last_m_date,
                &plant.plants_name,
                watering_point,
                current_weight,
                ldl,
            );

            result.push(fm);
        } else {
            if let Some(daily_loss) = daily_water_loss(&measurements) {
                let fm = format_plant_status(
                    last_m_date,
                    &plant.plants_name,
                    watering_point,
                    current_weight,
                    daily_loss,
                );
                result.push(fm);
            } else {
                let fm = format!("🪴 {} │ ⏳ Калибровка (нет замеров)", plant.plants_name);
                show_calibration_hint = true;
                result.push(fm);
                continue;
            }
        }
    }

    if show_calibration_hint {
        result.push("📌 Чтобы активировать прогноз для растений со статусом (⏳), взвесьте их один раз сразу после полива.".to_string());
    }

    Ok(result.join("\n─────────────\n"))
}

fn format_plant_status(
    last_m_date: DateTime<Utc>,
    plants_name: &str,
    watering_point: i64,
    current_weight: f32,
    daily_loss: f32,
) -> String {
    let days_since = (Utc::now().date_naive() - last_m_date.date_naive()).num_days();
    tracing::warn!(
        "plants_name{plants_name},current_weight {current_weight}, watering_point{watering_point} , daily_loss{daily_loss} "
    );
    let watering_day = match days_until_watering(current_weight as i64, watering_point, daily_loss)
    {
        Some(days) => days,
        None => return format!("🪴 {} │ ⚠️ ошибка данных", plants_name),
    };

    tracing::debug!("watering {}", watering_day);

    let days_untill = (watering_day - days_since).max(0) as f32;
    tracing::debug!("days_untill {}", days_untill);
    let status = watering_status(days_untill);

    let fm = format!("🪴 {} │  {}", plants_name, status);

    fm
}

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
            if let (Some(pct), Some(wet_weight), Some(dry_weight)) =
                (plant.threshold_pct, plant.wet_weight, plant.dry_weight)
            {
                format!(
                    "🌱 **{}**\n\
     🎯 Целевая влажность: {}%\n\
     ───\n\
     📊 **Калибровка веса:**\n\
     🟢 Мокрая почва: {} г\n\
     🟤 Сухая почва: {} г",
                    plant.plants_name,
                    pct.mul(100.0) as u8,
                    wet_weight,
                    dry_weight,
                )
            } else {
                format!(
                    "⚙️ Конфигурация полива для **{}** отсутствует\n\n\
     Создайте её, чтобы получать уведомления:\n\
     /start → Мои растения → Настроить полив",
                    plant.plants_name
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n─────────────\n");

    Ok(result)
}
