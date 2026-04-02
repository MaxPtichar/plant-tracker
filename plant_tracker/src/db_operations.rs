use crate::models::{Measurements, Plant, PlantWithLastFeedWatering, PotConfig, User};
use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Registers a user by their Telegram ID.
/// If the user already exists — does nothing (ON CONFLICT DO NOTHING).
pub async fn create_user(pool: &PgPool, tg_id: i64, username: Option<&str>) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO users (id, username) VALUES ($1, $2) ON CONFLICT(id) DO NOTHING",
        tg_id,
        username
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a new plant for the user.
/// Returns the id of the created plant.
pub async fn create_new_plant(
    pool: &PgPool,
    user_id: i64,
    plant_name: &str,
    target_moisture: f32,
) -> sqlx::Result<i64> {
    let result = sqlx::query!(
        "INSERT INTO plants (user_id, plants_name, target_moisture) VALUES ($1, $2, $3) RETURNING id ",
        user_id,
        plant_name,
        target_moisture
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}
/// Creates a new pot configuration for a plant.
/// Deactivates all previous configurations for this plant before inserting the new one.
pub async fn create_pot_config(
    pool: &PgPool,
    plant_id: i64,
    pot_w: i64,
    dry_w: i64,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE pot_configs SET is_active = false WHERE plant_id = $1",
        plant_id
    )
    .execute(pool)
    .await?;

    sqlx::query!("INSERT INTO pot_configs (plant_id, pot_weight, dry_soil_weight, is_active) VALUES($1, $2, $3, $4)", 
plant_id, pot_w, dry_w, true)
.execute(pool)
.await?;

    Ok(())
}

/// Returns the active pot configuration for a plant.
pub async fn get_active_config(pool: &PgPool, plant_id: i64) -> sqlx::Result<Option<PotConfig>> {
    sqlx::query_as!(
        PotConfig,
        "SELECT id, 
    plant_id, 
    pot_weight, 
    dry_soil_weight, 
    is_active
    FROM pot_configs WHERE plant_id = $1 AND is_active = true",
        plant_id
    )
    .fetch_optional(pool)
    .await
}

/// Returns all plants belonging to a user.
pub async fn get_user_plants(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<Plant>> {
    let plants = sqlx::query_as!(
        Plant,
        "SELECT id, user_id, plants_name, target_moisture FROM plants WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(plants)
}

/// Adds a new pot weight measurement for a plant.
pub async fn create_measurement(
    pool: &PgPool,
    plant_id: i64,
    weight: f32,
    date: NaiveDate,
    measuring_type: String,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO measurements (plant_id, weight, date, measuring_type) VALUES
    ($1, $2, $3, $4)",
        plant_id,
        weight,
        date,
        measuring_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Returns a user by their Telegram ID.
/// Returns None if the user is not registered.
pub async fn get_user(pool: &PgPool, tg_id: i64) -> sqlx::Result<Option<User>> {
    let res = sqlx::query_as!(
        User,
        "SELECT id, username, created_at FROM users WHERE id = $1",
        tg_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(res)
}

/// Returns the most recent AfterWatering measurement for a plant.
/// Returns None if no watering has been recorded yet.
pub async fn get_last_watering(pool: &PgPool, plant_id: i64) -> sqlx::Result<Option<Measurements>> {
    let res = sqlx::query_as!(
        Measurements,
        "SELECT id, plant_id, weight, date, measuring_type FROM measurements
        WHERE plant_id = $1 AND measuring_type = 'AfterWatering'
        ORDER BY date DESC LIMIT 1",
        plant_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(res)
}


//get last watering weight with watering status AfterWatering
pub async fn get_last_watering_weight(pool: &PgPool, plant_id: i64) -> sqlx::Result<Option<f32>> {
    let res = sqlx::query_scalar!(
        
        "SELECT weight FROM measurements
        WHERE plant_id = $1 AND measuring_type = 'AfterWatering'
        ORDER BY date DESC LIMIT 1",
        plant_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(res)
}




///return two last measurements of weight and date
pub async fn recieve_two_last_measurement(pool: &PgPool, plant_id: i64) -> sqlx::Result<Vec<Measurements>> {
    let res = sqlx::query_as!(
        Measurements,
        "SELECT id, plant_id, weight, date, measuring_type FROM measurements
        WHERE plant_id = $1 AND measuring_type = 'Regular'
        ORDER BY date DESC LIMIT 2",
        plant_id
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}



/// returning plants with dates when were last watering with feed
pub async fn recieve_plants_with_last_feed(
    pool: &PgPool,
    chat_id: i64,
) -> sqlx::Result<Vec<PlantWithLastFeedWatering>> {
    let res = sqlx::query_as!(
        PlantWithLastFeedWatering,
        "SELECT DISTINCT ON (p.id) 
    p.id as \"id!\",
    p.plants_name as \"plants_name!\",
    m.date as \"date?\"
FROM plants p
LEFT JOIN measurements m ON p.id = m.plant_id
AND m.measuring_type = 'AfterWateringWithFeed'
WHERE p.user_id = $1
ORDER BY p.id, m.date DESC",
        chat_id
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

/// Returns all measurements for a plant ordered by date.
pub async fn get_plant_measurements(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<Vec<Measurements>> {
    let measrements = sqlx::query_as!(
        Measurements,
        "SELECT id, plant_id,  weight, date, measuring_type FROM measurements WHERE plant_id = $1",
        plant_id
    )
    .fetch_all(pool)
    .await?;
    Ok(measrements)
}

/// Returns target_moisture dependes of plant id
pub async fn get_plant_target_moisture(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<f32> {
    let target_moisture = sqlx::query_scalar!(
        "SELECT target_moisture FROM plants WHERE id = $1",
        plant_id
    )
    .fetch_one(pool)
    .await?;
    Ok(target_moisture)
}




/// Deletes a plant and all its associated data (cascades to measurements and pot configs).
pub async fn delete_plant(pool: &PgPool, plant_id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM plants WHERE id = $1", plant_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Returns a single plant by its id.
pub async fn get_plant(pool: &PgPool, plant_id: i64) -> sqlx::Result<Plant> {
    let plant = sqlx::query_as!(
        Plant,
        "SELECT id, user_id, plants_name, target_moisture FROM plants WHERE id = $1",
        plant_id
    )
    .fetch_one(pool)
    .await?;

    Ok(plant)
}

/// Deletes the most recent measurement for a plant (used for /undo).
pub async fn delete_last_measurement(pool: &PgPool, plant_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM measurements WHERE id = (
    SELECT id FROM measurements WHERE plant_id = $1 ORDER BY date DESC LIMIT 1 )",
        plant_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
