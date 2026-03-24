use sqlx::FromRow;
use chrono::{DateTime, Utc};

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

}

#[derive(Debug, FromRow)] 
pub struct PotConfig {
    pub id: i64,
    pub plant_id: i64,
    pub pot_weight: i64,
    pub dry_soil_weight: i64, 
    pub is_active: bool

}