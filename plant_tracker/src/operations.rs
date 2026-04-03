use std::fmt::format;

// use crate::analytics::{days_until_watering, get_avg_r, last_plant_feed};

use crate::analytics::days_from_last_feed;
use crate::db_operations;
use crate::models::{Measurements, Plant, PlantWithLastFeedWatering};

use chrono::NaiveDate;

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
