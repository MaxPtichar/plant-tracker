use crate::models::{PlantMeasurementsHistory, PlantWithLastFeedWatering};
use chrono::NaiveDate;

/// Returns days elapsed since the last feed-watering measurement.
///
/// # Returns
/// - `Some(days)` — days since last `AfterWateringWithFeed`
/// - `None` — no feed-watering recorded yet
pub fn days_from_last_feed(last_feed: &PlantWithLastFeedWatering) -> Option<u32> {
    let date = last_feed.date?;
    let days = (chrono::Utc::now() - date).num_days();

    Some(days.max(0) as u32)
}
/// Вес при котором нужно поливать
/// threshold_pct - допустимый процент потери воды (0.0..1.0)
/// например 0.82 = поливать когда ушло 82% воды
pub fn watering_point(dry_weight: i64, wet_weight: i64, threshold_pct: f32) -> i64 {
    let capacity = (wet_weight - dry_weight) as f32;
    (dry_weight as f32 + capacity * (1.0 - threshold_pct)) as i64
}

/// Нужно ли поливать прямо сейчас
pub fn needs_watering(current_weight: i64, watering_point: i64) -> bool {
    current_weight <= watering_point
}

/// Дней до полива (None если нет данных по скорости убыли)
pub fn days_until_watering(
    current_weight: i64,
    watering_point: i64,
    daily_loss: f32,
) -> Option<i64> {
    let loss = daily_loss;
    if loss <= 0.0 {
        return None;
    }
    let days = ((current_weight - watering_point) as f32 / loss).max(0.0) as i64;

    Some(days)
}

/// Скорость убыли воды в граммах в день
/// after_watering - вес сразу после полива
/// next_regular   - следующее обычное взвешивание
pub fn daily_water_loss(plant: &[PlantMeasurementsHistory]) -> Option<f32> {
    let last_regular = plant.first().filter(|mt| mt.measuring_type == "Regular")?;

    let last_watering = plant.iter().find(|mt| {
        mt.measuring_type == "AfterWatering" || mt.measuring_type == "AfterWateringWithFeed"
    })?;

    let days = (last_regular.date - last_watering.date).num_days() as f32;

    if days <= 0.0 {
        return None;
    }

    Some((last_watering.weight - last_regular.weight) / days)
}


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
        return "🌱 Растений пока нет".to_string();
    }

    plants
        .iter()
        .map(|plant| match plant.date {
            Some(date) => format!(
                "🌿 {} \n   ┗ последняя подкормка: {} ({} дн. назад) 🫧 ",
                plant.plants_name,
                date.date_naive(),
                days_from_last_feed(plant).unwrap_or(0)
            ),
            None => format!("🪴 {} \n   ┗ подкормок не было 🫙", plant.plants_name,),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
