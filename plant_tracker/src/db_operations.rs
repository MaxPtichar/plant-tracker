use crate::models::{
    Measurements, Plant, PlantDetails, PlantMeasurementsHistory, PlantWithLastFeedWatering, User,
};

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
pub async fn create_new_plant(pool: &PgPool, user_id: i64, plant_name: &str) -> sqlx::Result<i64> {
    let result = sqlx::query!(
        "INSERT INTO plants (user_id, plants_name)
         VALUES ($1, $2) RETURNING id",
        user_id,
        plant_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}
/// Creates a new Water configuration for a plant.
/// Deactivates all previous configurations for this plant before inserting the new one.
pub async fn create_watering_config(
    pool: &PgPool,
    plant_id: i64,
    wet_weight: i64,
    dry_weight: i64,
    threshold_pct: f32,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE watering_config SET is_active = false WHERE plant_id = $1",
        plant_id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "INSERT INTO watering_config
         (plant_id, wet_weight, dry_weight, threshold_pct, is_active)
         VALUES ($1, $2, $3, $4, true)",
        plant_id,
        wet_weight,
        dry_weight,
        threshold_pct,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_daily_loss_to_null(pool: &PgPool, plant_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE watering_config SET learned_daily_loss = NULL WHERE plant_id = $1 AND is_active = true",
        plant_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_daily_loss(pool: &PgPool, plant_id: i64) -> sqlx::Result<Option<f32>> {
    let res = sqlx::query!(
        "SELECT learned_daily_loss FROM watering_config WHERE 
    plant_id = $1 AND is_active = true  ",
        plant_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(daily_loss) = res {
        return Ok(daily_loss.learned_daily_loss);
    }

    Ok(None)
}

pub async fn update_daily_loss(pool: &PgPool, plant_id: i64, daily_loss: f32) -> sqlx::Result<()> {
    sqlx::query!(
        "
    UPDATE watering_config SET learned_daily_loss = $1 WHERE plant_id = $2 AND is_active = true
    ",
        daily_loss,
        plant_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Returns all plants belonging to a user.
pub async fn get_user_plants(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<Plant>> {
    sqlx::query_as!(
        Plant,
        "SELECT 
            id, 
            plants_name
         FROM plants 
         WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await
}

/// Adds a new pot weight measurement for a plant.
pub async fn create_measurement(
    pool: &PgPool,
    plant_id: i64,
    weight: f32,
    date: chrono::DateTime<chrono::Utc>, // СТРОГО DateTime<Utc>
    measuring_type: String,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO measurements (plant_id, weight, date, measuring_type) VALUES ($1, $2, $3, $4)",
        plant_id,
        weight,
        date, // Теперь они совпадут!
        measuring_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Returns all registered users.
/// Used by the notification system to iterate over all chat IDs.
pub async fn get_all_users(pool: &PgPool) -> sqlx::Result<Vec<User>> {
    let res = sqlx::query_as!(User, "SELECT id FROM users",)
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

/// Deletes a plant and all its associated data (cascades to measurements and pot configs).
pub async fn delete_plant(pool: &PgPool, plant_id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM plants WHERE id = $1", plant_id)
        .execute(pool)
        .await?;

    Ok(())
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

/// Returns plant details with active pot config and last measurement date for all user's plants.
/// Used by [`get_list_of_all_plants`] to render the plant list screen.
///
///
///
pub async fn get_list_of_all_user_plants(
    pool: &PgPool,
    chat_id: i64,
) -> sqlx::Result<Vec<PlantDetails>> {
    sqlx::query_as::<_, PlantDetails>(
        "SELECT 
        p.id,
        p.plants_name, 
        w.wet_weight,
        w.dry_weight, 
        COALESCE(w.learned_threshold_pct, w.threshold_pct) AS threshold_pct,
         w.learned_daily_loss
FROM plants p
LEFT JOIN watering_config w ON p.id = w.plant_id AND w.is_active = true
WHERE p.user_id = $1",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
}

pub async fn check_water_config(pool: &PgPool, plant_id: i64) -> sqlx::Result<bool> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM watering_config WHERE plant_id = $1 )",
        plant_id
    )
    .fetch_one(pool)
    .await?;

    Ok(exists.unwrap_or(false))
}

/// Returns the last 20 measurements for a plant, newest first.
/// Filters by both `plant_id` and `chat_id` to prevent access to another user's data.
pub async fn get_measurement_record_20(
    pool: &PgPool,
    plant_id: i64,
    chat_id: i64,
) -> sqlx::Result<Vec<PlantMeasurementsHistory>> {
    sqlx::query_as!(
        PlantMeasurementsHistory,
        "SELECT m.weight, m.date, m.measuring_type 
FROM measurements m
LEFT JOIN plants p ON p.id = m.plant_id
WHERE p.id = $1 AND p.user_id = $2
ORDER BY m.date DESC, m.id DESC
LIMIT 30;",
        plant_id,
        chat_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_last_after_watering(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<Option<(f32, DateTime<Utc>)>> {
    let res = sqlx::query!(
        "
    SELECT weight, date
    FROM measurements
    WHERE plant_id = $1 AND
    (measuring_type = 'AfterWatering' OR 
    measuring_type = 'AfterWateringWithFeed')
    ORDER BY date DESC
    LIMIT 1
    
    ",
        plant_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = res {
        return Ok(Some((row.weight, row.date)));
    }

    Ok(None)
}

pub async fn get_last_regular_before_watering(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<Option<Measurements>> {
    sqlx::query_as!(
        Measurements,
        "SELECT weight, date
        FROM measurements
        WHERE plant_id = $1
          AND measuring_type = 'Regular'
          AND date < COALESCE(
              (SELECT MAX(date) FROM measurements
               WHERE plant_id = $1
                 AND (measuring_type = 'AfterWatering'
                      OR measuring_type = 'AfterWateringWithFeed')),
              '1970-01-01'
          )
        ORDER BY date DESC LIMIT 1",
        plant_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_last_measurement(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<Option<(f32, DateTime<Utc>)>> {
    let res = sqlx::query!(
        "SELECT weight, date FROM measurements WHERE plant_id = $1 ORDER BY date DESC LIMIT 1 ",
        plant_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(data) = res {
        Ok(Some((data.weight, data.date)))
    } else {
        Ok(None)
    }
}
