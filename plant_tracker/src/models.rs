use core::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy)]
pub enum MeasurementType {
    Regular,
    AfterWatering,
    AfterWateringWithFeed,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64, //chat_id in telegram
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct Plant {
    pub id: i64,
    pub user_id: i64,
    pub plants_name: String,
    pub target_moisture: f32,
}

#[derive(Debug, FromRow)]
pub struct PotConfig {
    pub id: i64,
    pub plant_id: i64,
    pub pot_weight: i64,
    pub dry_soil_weight: i64,
    pub is_active: bool,
}

#[derive(Debug, FromRow)]
pub struct Measurements {
    pub id: i64,
    pub plant_id: i64,
    pub weight: f32,
    pub date: NaiveDate,
    pub measuring_type: String,
}

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
