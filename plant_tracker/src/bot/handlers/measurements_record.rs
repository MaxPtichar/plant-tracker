use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue, PlantCreationDialogue,
        keyboards::{back_to_my_plants, get_type_of_moisture, main_menu_buttons},
    },
    db_operations,
};

/// Handles plant selection for viewing measurement history.
///
/// Parses `plant_id` and `plant_name` from callback data (`"id:name"`),
/// fetches the last 20 measurements and sends them to the user.
pub async fn receive_plant_for_record(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    if let Some(data) = q.data {
        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();

        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;

        let text = get_list_of_measurements_20(&pool, chat_id.0, plant_id).await?;

        bot.send_message(
            chat_id,
            format!("☘️ {plant_name}\n\nИстория последних 20 измерений:\n\n{text}"),
        )
        .reply_markup(back_to_my_plants())
        .await?;

        dialogue.exit().await?;
    }
    Ok(())
}

/// Fetches and formats the last 20 measurements for a plant.
///
/// # Returns
/// - Formatted string with weight, date and measurement type
/// - `"Нет измерений"` if no records found
pub async fn get_list_of_measurements_20(
    pool: &PgPool,
    chat_id: i64,
    plant_id: i64,
) -> sqlx::Result<String> {
    let measurements = db_operations::get_measurement_record_20(pool, plant_id, chat_id).await?;

    if measurements.is_empty() {
        return Ok("Нет измерений 📊".to_string());
    }

    let result = measurements
        .iter()
        .map(|m| {
            format!(
                "⚖️ {} г  📅 {}  🔬 {}",
                m.weight,
                m.date,
                m.measuring_type,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(result)
}