use core::fmt;

use chrono::{DateTime, Utc};
use sqlx::FromRow;
/// Measurement type stored in the `measurements` table.
///
/// Determines the context in which a weight measurement was taken,
/// which affects how moisture calculations are performed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeasurementType {
    /// Routine weight check, no watering or feeding performed.
    Regular,
    /// Weight recorded immediately after watering.
    AfterWatering,
    /// Weight recorded after watering with liquid fertilizer.
    AfterWateringWithFeed,
}

/// Converts a `String` from the database into [`MeasurementType`].
/// Unknown values default to [`MeasurementType::Regular`].
impl From<String> for MeasurementType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Regular" => Self::Regular,
            "AfterWatering" => Self::AfterWatering,
            "AfterWateringWithFeed" => Self::AfterWateringWithFeed,
            _ => Self::Regular,
        }
    }
}

/// Serializes [`MeasurementType`] back to the string representation
/// used in the database and Telegram callback data.
impl fmt::Display for MeasurementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Regular => "Regular",
            Self::AfterWatering => "AfterWatering",
            Self::AfterWateringWithFeed => "AfterWateringWithFeed",
        };
        write!(f, "{}", s)
    }
}

pub fn format_measurement_type(m_type: &str) -> &str {
    match m_type {
        "Regular" => "Обычный вес",
        "AfterWatering" => "После полива",
        "AfterWateringWithFeed" => "С прикормкой",
        _ => "❓ Неизвестный тип", // На случай, если в базе появится что-то новое
    }
}

/// Query result joining `plants` and `measurements`.
///
/// Used to display the last feed-watering date per plant.
/// `date` is `None` if no `AfterWateringWithFeed` measurement exists yet.
#[derive(Debug, FromRow)]
pub struct PlantWithLastFeedWatering {
    pub plants_name: String,
    pub date: Option<DateTime<Utc>>,
}

/// Telegram user identified by `chat_id`.
#[derive(Debug, FromRow)]
pub struct User {
    /// Telegram chat ID, used as the primary identifier.
    pub id: i64,
}

#[derive(Debug, FromRow)]
pub struct Plant {
    pub id: i64,
    pub plants_name: String,
}

/// A single weight measurement for a plant.
#[derive(Debug, FromRow, Clone)]
pub struct Measurements {
    /// Measured weight of the pot in grams.
    pub weight: f32,
    pub date: DateTime<Utc>,
}

/// Represents the urgency of the next watering for a plant.
///
/// Variants are ordered from most to least urgent:
/// - [`Overdue`] — watering is overdue (days < 0)
/// - [`Urgent`] — must water today (0 ≤ days < 1)
/// - [`Soon`] — water within the next few days (1 ≤ days < 3)
/// - [`Wait`] — no action needed yet (days ≥ 3)
pub enum WateringStatus {
    /// Watering deadline has passed.
    Overdue,
    /// Watering is due today.
    Urgent,
    /// Watering is due soon. Contains days remaining as `f32`.
    Soon(f32),
    /// No watering needed yet. Contains days remaining as `f32`.
    Wait(f32),
}

/// Returns the [`WateringStatus`] for a given number of days until next watering.
///
/// # Thresholds
/// - `days < 0.0` → [`WateringStatus::Overdue`]
/// - `days < 1.0` → [`WateringStatus::Urgent`]
/// - `days < 3.0` → [`WateringStatus::Soon`]
/// - `days ≥ 3.0` → [`WateringStatus::Wait`]
pub fn watering_status(days: f32) -> WateringStatus {
    match days {
        d if d < 0.0 => WateringStatus::Overdue,
        d if d < 1.0 => WateringStatus::Urgent,
        d if d < 3.0 => WateringStatus::Soon(d),
        d => WateringStatus::Wait(d),
    }
}
/// Returns the correct Russian declension of "день" for a given number of days.
///
/// Handles the special case of 11–14 (e.g. "11 дней", not "11 дня"),
/// then applies standard rules for the last digit.
///
/// # Examples
/// ```
/// assert_eq!(watering_status_days_name(1.0), "день");
/// assert_eq!(watering_status_days_name(3.0), "дня");
/// assert_eq!(watering_status_days_name(11.0), "дней");
/// assert_eq!(watering_status_days_name(21.0), "день");
/// ```
fn watering_status_days_name(days: f32) -> &'static str {
    let days_round = days.floor() as u32;
    if (11..=14).contains(&(days_round % 100)) {
        "дней"
    } else {
        match days_round % 10 {
            1 => "день",
            2..=4 => "дня",
            _ => "дней",
        }
    }
}
/// Formats [`WateringStatus`] as a human-readable Russian string with emoji.
///
/// # Output examples
/// ```text
/// Overdue      → "🚨 Полив просрочен!"
/// Urgent       → "⚠️  Полить сегодня!"
/// Soon(2.0)    → "🕐 Полить в ближайшие дни - 2 дня до полива"
/// Wait(5.0)    → "✅ Ещё 5 дней"
/// ```
impl fmt::Display for WateringStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WateringStatus::Overdue => write!(f, "🚨 Полив просрочен!"),
            WateringStatus::Urgent => write!(f, "⚠️  Полить сегодня!"),
            WateringStatus::Soon(days) => write!(
                f,
                "🕐 Полить в ближайшие дни - {} {} до полива",
                days.floor() as u32,
                watering_status_days_name(*days)
            ),
            WateringStatus::Wait(days) => write!(
                f,
                "✅ Ещё {} {}",
                days.floor(),
                watering_status_days_name(*days)
            ),
        }
    }
}

/// Plant details joined with active pot config and last measurement date.
/// Used to render the plant list screen ([`get_list_of_all_user_plants`]).
///
#[derive(Debug, FromRow)]
pub struct PlantDetails {
    pub id: i64,
    pub plants_name: String,
    /// Weight of the empty pot in grams. `None` if no pot config exists.
    pub wet_weight: Option<i64>,
    /// Weight of fully dry soil in grams. `None` if no pot config exists.
    pub dry_weight: Option<i64>,
    /// Date of the most recent measurement of any type.
    /// `None` if no measurements have been recorded yet.
    pub threshold_pct: Option<f32>,

    pub learned_daily_loss: Option<f32>,
}

/// A single measurement record joined with the plant name.
/// Used to render the measurement history screen ([`get_measurement_record_20`]).
#[derive(Debug, FromRow)]
pub struct PlantMeasurementsHistory {
    /// Measured weight of the pot in grams.
    pub weight: f32,
    pub date: DateTime<Utc>,
    /// String representation of [`MeasurementType`].
    pub measuring_type: String,
}
