use core::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::constants::{
    CHLOROPHYTUM_DRY, CHLOROPHYTUM_THRESHOLD, FICUS_BLACK_PRINCE_DRY, FICUS_BLACK_PRINCE_THRESHOLD,
    FICUS_KINKY_DRY, FICUS_KINKY_THRESHOLD, FICUS_MICROCARPA_DRY, FICUS_MICROCARPA_THRESHOLD,
    R_CHLOROPHYTUM, R_FICUS_BLACK_PRINCE, R_FICUS_KINKY, R_FICUS_MICROCARPA, R_SANSEVIERIA,
    SAFE_DAYS_CHLOROPHYTUM, SAFE_DAYS_FICUS_BLACK_PRINCE, SAFE_DAYS_FICUS_KINKY,
    SAFE_DAYS_FICUS_MICROCARPA, SAFE_DAYS_SANSEVIERIA, SANSEVIERIA_DRY, SANSEVIERIA_THRESHOLD,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum PlantType {
    Chlorophytum,
    FicusKinki,
    FicusMicrocarpa,
    FicusBlackPrince,
    Sansevieria,
}

pub trait PlantBehavoir {
    fn avg_water_loss_per_day(&self) -> f32;
    fn avg_days_beetwen_watering(&self) -> f32;
    fn watering_threshold(&self) -> f32;
    fn dry_weight(&self) -> f32;
}

impl PlantBehavoir for PlantType {
    fn avg_water_loss_per_day(&self) -> f32 {
        match self {
            PlantType::Chlorophytum => R_CHLOROPHYTUM,
            PlantType::FicusBlackPrince => R_FICUS_BLACK_PRINCE,
            PlantType::FicusKinki => R_FICUS_KINKY,
            PlantType::FicusMicrocarpa => R_FICUS_MICROCARPA,
            PlantType::Sansevieria => R_SANSEVIERIA,
        }
    }

    fn avg_days_beetwen_watering(&self) -> f32 {
        match self {
            PlantType::Chlorophytum => SAFE_DAYS_CHLOROPHYTUM,
            PlantType::FicusBlackPrince => SAFE_DAYS_FICUS_BLACK_PRINCE,
            PlantType::FicusKinki => SAFE_DAYS_FICUS_KINKY,
            PlantType::FicusMicrocarpa => SAFE_DAYS_FICUS_MICROCARPA,
            PlantType::Sansevieria => SAFE_DAYS_SANSEVIERIA,
        }
    }

    fn watering_threshold(&self) -> f32 {
        match self {
            PlantType::Chlorophytum => CHLOROPHYTUM_THRESHOLD,
            PlantType::FicusBlackPrince => FICUS_BLACK_PRINCE_THRESHOLD,
            PlantType::FicusKinki => FICUS_KINKY_THRESHOLD,
            PlantType::FicusMicrocarpa => FICUS_MICROCARPA_THRESHOLD,
            PlantType::Sansevieria => SANSEVIERIA_THRESHOLD,
        }
    }

    fn dry_weight(&self) -> f32 {
        match self {
            PlantType::Chlorophytum => CHLOROPHYTUM_DRY,
            PlantType::FicusBlackPrince => FICUS_BLACK_PRINCE_DRY,
            PlantType::FicusKinki => FICUS_KINKY_DRY,
            PlantType::FicusMicrocarpa => FICUS_MICROCARPA_DRY,
            PlantType::Sansevieria => SANSEVIERIA_DRY,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]

pub enum MeasurementType {
    Regular,
    AfterWatering,
    AfterWateringWithFeed,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct Measurement {
    pub weight: f32,
    pub date: NaiveDate,
    pub type_: MeasurementType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plant {
    pub id: u32,
    pub name: String,
    pub plant_type: PlantType,
    pub measurements: Vec<Measurement>,
    pub avg_r: Option<f32>,
}

impl Plant {
    pub fn new(
        id: u32,
        name: String,
        plant_type: PlantType,
        measurements: Vec<Measurement>,
        avg_r: Option<f32>,
    ) -> Self {
        Self {
            id,
            name,
            plant_type,
            measurements,
            avg_r: None,
        }
    }

    pub fn update_avg_r(&mut self, avg: Option<f32>) {
        self.avg_r = avg;
    }

    pub fn print_plants(plants: &Vec<Plant>) {
        println!("{:?}", plants)
    }
}

pub enum WateringStatus {
    Overdue,   // просрочен — меньше 0 дней
    Urgent,    // срочно — меньше 1 дня
    Soon,      // скоро — 1-2 дня
    Wait(f32), // подождать — больше 2 дней, храним сколько именно
}

impl fmt::Display for WateringStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WateringStatus::Overdue => write!(f, "🚨 Полив просрочен!"),
            WateringStatus::Urgent => write!(f, "⚠️  Полить сегодня!"),
            WateringStatus::Soon => write!(f, "🕐 Полить в ближайшие дни"),
            WateringStatus::Wait(days) => write!(f, "✅ Ещё {} дней", days.round()),
        }
    }
}

pub fn watering_status(days: f32) -> WateringStatus {
    match days {
        d if d < 0.0 => WateringStatus::Overdue,
        d if d < 1.0 => WateringStatus::Urgent,
        d if d < 3.0 => WateringStatus::Soon,
        d => WateringStatus::Wait(d),
    }
}
