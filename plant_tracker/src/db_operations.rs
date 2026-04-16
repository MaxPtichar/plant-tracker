use crate::models::{
    Measurements, Plant, PlantDetails, PlantFullContext, PlantMeasurementsHistory,
    PlantWithLastFeedWatering, PotConfig, User, UserGeo,
};
use chrono::NaiveDate;
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
    plant_type: &str,
    light_level: &str,
    air_circulation: &str,
) -> sqlx::Result<i64> {
    let result = sqlx::query!(
        "INSERT INTO plants (user_id, plants_name, plant_type, light_level, air_circulation)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
        user_id,
        plant_name,
        plant_type,
        light_level,
        air_circulation,
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
    pot_diameter_cm: f32,
    soil_type: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE pot_configs SET is_active = false WHERE plant_id = $1",
        plant_id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        "INSERT INTO pot_configs 
         (plant_id, pot_weight, dry_soil_weight, is_active, pot_diameter_cm, soil_type)
         VALUES ($1, $2, $3, true, $4, $5)",
        plant_id,
        pot_w,
        dry_w,
        pot_diameter_cm,
        soil_type
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns the active pot configuration for a plant.
pub async fn get_active_config(pool: &PgPool, plant_id: i64) -> sqlx::Result<Option<PotConfig>> {
    sqlx::query_as!(
        PotConfig,
        "SELECT id, plant_id, pot_weight, dry_soil_weight, is_active,
         pot_diameter_cm, soil_type
         FROM pot_configs WHERE plant_id = $1 AND is_active = true",
        plant_id
    )
    .fetch_optional(pool)
    .await
}

/// Returns all plants belonging to a user.
pub async fn get_user_plants(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<Plant>> {
    sqlx::query_as!(
        Plant,
        "SELECT id, user_id, plants_name,
         plant_type, light_level, air_circulation,
         transpiration_coef, avg_r, cycles_count
         FROM plants WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await
}

//return all data that plant have
pub async fn get_all_plant_data(
    pool: &PgPool,
    user_id: i64,
    plant_id: i64,
) -> sqlx::Result<PlantFullContext> {
    sqlx::query_as!(
        PlantFullContext,
        r#"
    SELECT 
        p.id AS "plant_id!",
        p.plants_name AS "plants_name!",
        p.plant_type AS "plant_type!",
        p.light_level AS "light_level!",
        p.air_circulation AS "air_circulation!",
        p.transpiration_coef AS "transpiration_coef!",
        p.avg_r,
        p.cycles_count AS "cycles_count!",
        pc.pot_weight AS "pot_weight!",
        pc.dry_soil_weight AS "dry_soil_weight!",
        pc.pot_diameter_cm AS "pot_diameter_cm!",
        pc.soil_type AS "soil_type!",
        -- Добавлена запятая перед вторым подзапросом
        (SELECT weight FROM measurements 
         WHERE plant_id = p.id AND measuring_type = 'Regular'
         ORDER BY date DESC LIMIT 1) AS "current_weight",
         
       (SELECT weight FROM measurements 
 WHERE plant_id = p.id 
   AND (measuring_type = 'AfterWatering' OR measuring_type = 'AfterWateringWithFeed')
 ORDER BY date DESC LIMIT 1) AS "last_watering_weight",

         (SELECT date FROM measurements 
     WHERE plant_id = p.id AND (measuring_type = 'AfterWatering' or measuring_type = 'AfterWateringWithFeed')
     ORDER BY date DESC LIMIT 1) AS "last_watering_date"
    FROM plants p
    JOIN pot_configs pc ON p.id = pc.plant_id
    WHERE p.user_id = $1 AND plant_id = $2 AND pc.is_active = true
    LIMIT 1; 
    "#,
        user_id,
        plant_id
    )
    .fetch_one(pool)
    .await
}


//return data for all plants that user have
pub async fn get_all_plants_data(
    pool: &PgPool,
    user_id: i64,
) -> sqlx::Result<Vec<PlantFullContext>> {
    sqlx::query_as!(
        PlantFullContext,
        r#"
    SELECT 
        p.id AS "plant_id!",
        p.plants_name AS "plants_name!",
        p.plant_type AS "plant_type!",
        p.light_level AS "light_level!",
        p.air_circulation AS "air_circulation!",
        p.transpiration_coef AS "transpiration_coef!",
        p.avg_r,
        p.cycles_count AS "cycles_count!",
        pc.pot_weight AS "pot_weight!",
        pc.dry_soil_weight AS "dry_soil_weight!",
        pc.pot_diameter_cm AS "pot_diameter_cm!",
        pc.soil_type AS "soil_type!",
        -- Добавлена запятая перед вторым подзапросом
        (SELECT weight FROM measurements 
         WHERE plant_id = p.id AND measuring_type = 'Regular'
         ORDER BY date DESC LIMIT 1) AS "current_weight",
        (SELECT weight FROM measurements 
 WHERE plant_id = p.id 
   AND (measuring_type = 'AfterWatering' OR measuring_type = 'AfterWateringWithFeed')
 ORDER BY date DESC LIMIT 1) AS "last_watering_weight",

         (SELECT date FROM measurements 
     WHERE plant_id = p.id AND (measuring_type = 'AfterWatering' or measuring_type = 'AfterWateringWithFeed') 
     ORDER BY date DESC LIMIT 1) AS "last_watering_date"
    FROM plants p
    JOIN pot_configs pc ON p.id = pc.plant_id
    WHERE p.user_id = $1 AND pc.is_active = true; 
    "#,
        user_id,
        
    )
    .fetch_all(pool)
    .await
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

/// Returns all registered users.
/// Used by the notification system to iterate over all chat IDs.
pub async fn get_all_users(pool: &PgPool) -> sqlx::Result<Vec<User>> {
    let res = sqlx::query_as!(User, "SELECT id, username, created_at FROM users",)
        .fetch_all(pool)
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

/// Returns the weight of the most recent `AfterWatering` measurement for a plant.
/// Used as the `AfterWatering_weight` parameter in watering calculations.
/// Returns `None` if no watering has been recorded yet.
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

/// Returns the last 2 `Regular` measurements for a plant, newest first.
/// Used as input for evaporation rate calculation in [`analytics::avg_evaporation_rate`].
pub async fn recieve_two_last_measurement(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<Vec<Measurements>> {
    let res = sqlx::query_as!(
        Measurements,
        "SELECT id, plant_id, weight, date, measuring_type FROM measurements
        WHERE plant_id = $1 AND measuring_type = 'Regular'
        ORDER BY date DESC, id DESC LIMIT 2",
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

/// Deletes a plant and all its associated data (cascades to measurements and pot configs).
pub async fn delete_plant(pool: &PgPool, plant_id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM plants WHERE id = $1", plant_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Returns a single plant by its id.
pub async fn get_plant(pool: &PgPool, plant_id: i64) -> sqlx::Result<Plant> {
    sqlx::query_as!(
        Plant,
        "SELECT id, user_id, plants_name,
         plant_type, light_level, air_circulation,
         transpiration_coef, avg_r, cycles_count
         FROM plants WHERE id = $1",
        plant_id
    )
    .fetch_one(pool)
    .await
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

//переписать эту функци.
pub async fn get_list_of_all_user_plants(
    pool: &PgPool,
    chat_id: i64,
) -> sqlx::Result<Vec<PlantDetails>> {
    sqlx::query_as!(
        PlantDetails,
        "SELECT p.plants_name, pot.pot_weight, pot.dry_soil_weight,
MAX(m.date) as last_measurement_date FROM plants p 
LEFT JOIN measurements m ON p.id = m.plant_id
LEFT JOIN pot_configs pot ON p.id = pot.plant_id AND pot.is_active = true
WHERE p.user_id = $1
GROUP BY p.id, p.plants_name, pot.pot_weight, pot.dry_soil_weight",
        chat_id
    )
    .fetch_all(pool)
    .await
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
        "SELECT p.plants_name, m.weight, m.date, m.measuring_type 
FROM measurements m
LEFT JOIN plants p ON p.id = m.plant_id
WHERE p.id = $1 AND p.user_id = $2
ORDER BY m.date, m.id DESC
LIMIT 20;",
        plant_id,
        chat_id
    )
    .fetch_all(pool)
    .await
}

/// Updates avg_r, transpiration_coef and cycles_count after a completed watering cycle.
/// Called when a new AfterWatering measurement is added.
pub async fn update_plant_after_cycle(
    pool: &PgPool,
    plant_id: i64,
    new_avg_r: f32,
    new_transpiration_coef: f32,
) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE plants SET
         avg_r = $1,
         transpiration_coef = $2,
         cycles_count = cycles_count + 1
         WHERE id = $3",
        new_avg_r,
        new_transpiration_coef,
        plant_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns Regular measurements after the last AfterWatering, newest first.
/// Used for current cycle evaporation rate calculation.
pub async fn get_regular_after_last_watering(
    pool: &PgPool,
    plant_id: i64,
) -> sqlx::Result<Vec<Measurements>> {
    sqlx::query_as!(
        Measurements,
        "SELECT id, plant_id, weight, date, measuring_type
FROM measurements
WHERE plant_id = $1
  AND measuring_type = 'Regular'
  AND date > COALESCE(
      (SELECT MAX(date) FROM measurements
       WHERE plant_id = $1
         AND (measuring_type = 'AfterWatering'
              OR measuring_type = 'AfterWateringWithFeed')),
      '1970-01-01'
  )
ORDER BY date DESC, id DESC",
        plant_id
    )
    .fetch_all(pool)
    .await
}




pub async fn get_geo_data(pool: &PgPool, user_id: i64) -> sqlx::Result<Option<UserGeo>> {
    let res = sqlx::query_as!(UserGeo, 
        "SELECT latitude, longitude
        FROM users
        WHERE id = $1",
        user_id)
    .fetch_optional(pool)
    .await?;

    Ok(res)
}


pub async fn create_geo(pool: &PgPool,latitude: f64, longitude: f64, user_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE users SET
        latitude = $1, 
        longitude = $2 
        WHERE id = $3",
        latitude, longitude, user_id)
    .execute(pool)
    .await?;
    Ok(())


}