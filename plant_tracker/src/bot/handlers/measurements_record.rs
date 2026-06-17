use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{HandlerResult, MeasurementDialogue, MyDialogue, keyboards::back_to_my_plants},
    db_operations, models::format_measurement_type,
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
            format!("☘️ {plant_name}\n\n{text}"),
        )
        .reply_markup(back_to_my_plants())
        .await?;

        dialogue
            .update(MeasurementDialogue::WaitingForPlantRecord)
            .await?;
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
    let table_rows: Vec<String> = measurements
        .iter()
        .take(10)
        .map(|m| {
            format!(
                "{} │ {:<21} │{:>4} г",
                m.date.format("%d.%m"),
                format_measurement_type(&m.measuring_type),
                m.weight
            )
        })
        .collect();

    let result = format!(
        "История замеров (последние 10):\n\n \n{}\n",
        table_rows.join("\n")
    );

    Ok(result)
}
