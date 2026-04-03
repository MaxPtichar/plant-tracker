use sqlx::PgPool;

use crate::{analytics::days_until_watering, db_operations, models::watering_status};

///get status for all plants
/// status answer an question: wheh is should watering plants? it is returning days untill watering
pub async fn get_all_plants_status(pool: &PgPool, chat_id: i64) -> sqlx::Result<String> {
    let plants = db_operations::get_user_plants(&pool, chat_id).await?;
    if plants.is_empty() {
        return Ok(format!("Пока еще нет ни одного растения🌱"));
    }
    let mut result: Vec<String> = Vec::new();

    for plant in &plants {
        let measurements = db_operations::recieve_two_last_measurement(&pool, plant.id).await?;
        let pot = db_operations::get_active_config(&pool, plant.id).await?;
        let target_moisture = plant.target_moisture;
        let after_watering_weight =
            db_operations::get_last_watering_weight(&pool, plant.id).await?;
        dbg!(&pot);
        dbg!(&after_watering_weight);

        let line = match (pot, after_watering_weight) {
            (Some(pot), Some(after_watering_weight)) => {
                match days_until_watering(
                    &measurements,
                    &pot,
                    target_moisture,
                    after_watering_weight,
                ) {
                    Some(days) => format!("🌱 {} — {}", plant.plants_name, watering_status(days)),
                    None => format!("🌱 {} — нет данных", plant.plants_name),
                }
            }

            _ => format!("🌱 {} — не настроено", plant.plants_name),
        };

        result.push(line);
    }

    Ok(result.join("\n"))
}

pub async fn get_list_of_all_plants(pool: &PgPool, chat_id: i64) -> sqlx::Result<String> {
    let plants = db_operations::get_list_of_all_user_plants(pool, chat_id).await?;
    if plants.is_empty() {
        return Ok(format!("Пока еще нет ни одного растения🌱"));
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
                (plant.target_moisture * 100.0).round(),
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
