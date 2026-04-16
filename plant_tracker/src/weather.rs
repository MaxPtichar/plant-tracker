use std::time::Duration;

use crate::{
    analytics_new::set_outdoor_temp, constants::T_INDOOR_BASE, db_operations::{get_all_users_geo, get_geo_data},
    models::WeatherResponse,
};
use reqwest;
use sqlx::{PgPool, pool};
use tokio::time::{self, error};

async fn get_weather(lat: f64, long: f64) -> Result<f32, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m&timezone=auto&forecast_days=1",
        lat, long
    );
    let body = reqwest::get(url).await?.json::<WeatherResponse>().await?;

    Ok(body.current.temperature_2m)
}

pub async fn get_weather_scheldue(pool: PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let durataion = Duration::from_secs(3 * 3600);
    let mut interval = time::interval(durataion);

    loop {
        interval.tick().await;
        let users_geo= get_all_users_geo(&pool).await?;
        
        for user_geo in users_geo {
            let (lat, long) = match (user_geo.latitude, user_geo.longitude) {
                (Some(l), Some(lg)) => (l, lg),
                _ => continue,
            };
             let temp = match get_weather(lat, long).await {
                Ok(temp) => temp as f32,
                Err(e) => {
                    eprintln!("Ошибка API погоды: {}", e);
                    T_INDOOR_BASE
                }
        };
        set_outdoor_temp(temp).await;

    }
}
}