use sqlx::PgPool;
use teloxide::prelude::*;

use crate::models::MeasurementType;
use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue,
        callbacks::{parse_date, parse_measurement_type},
        keyboards::{date_keyboard, measurement_type_keyboard, plant_keyboard},
    },
    db_operations,
};

// ============================================================
// Measurement recording — multi-step dialogue
//
// Drives a four-state FSM that collects weight, measurement
// type, and date for a selected plant, then persists the result.
//
// States (MeasurementDialogue):
//   WaitingForPlant   → expects a callback (plant selection)
//   WaitingForWeight  → expects a text message (float)
//   WaitingForType    → expects a callback (measurement type)
//   WaitingForDate    → expects a callback (date selection)
//
// Handler routing:
//   callback  + WaitingForPlant              → receive_plant
//   text msg  + WaitingForWeight             → receive_weight
//   callback  + WaitingForType  { .. }       → receive_type
//   callback  + WaitingForDate  { .. }       → receive_date
// ============================================================

/// **Stage 0 of 3** — selects the target plant from the inline keyboard.
///
/// Triggered by: callback query while in `WaitingForPlant`.
///
/// On success: advances state to `WaitingForWeight { plant_id }`
/// and prompts the user to enter weight in grams.
///
/// Ignores non-numeric or missing callback data silently.
pub async fn receive_plant(bot: Bot, q: CallbackQuery, dialogue: MyDialogue) -> HandlerResult {
    if let Some(data) = q.data {
        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();

        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(
            chat_id,
            format!("☘️ {plant_name}\n\nВведите текущий вес горшка в граммах:"),
        )
        .await?;

        dialogue
            .update(MeasurementDialogue::WaitingForWeight {
                plant_id,
                plant_name,
            })
            .await?;
    }
    Ok(())
}

/// **Stage 1 of 3** — collects the plant's weight in grams.
///
/// Triggered by: text message while in `WaitingForWeight`.
///
/// On success: advances state to `WaitingForType { plant_id, weight }`
/// and sends the measurement type keyboard.
///
/// On failure: replies with an error and stays in the current state.
pub async fn receive_weight(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (plant_id, plant_name): (i64, String),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<f32>() {
            Ok(weight) => {
                dialogue
                    .update(MeasurementDialogue::WaitingForType { plant_id, weight })
                    .await?;

                bot.send_message(msg.chat.id, "Выбери тип измерений: ")
                    .reply_markup(measurement_type_keyboard())
                    .await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Введите число").await?;
            }
        },

        None => {
            bot.send_message(msg.chat.id, "Введите вес в граммах")
                .await?;
        }
    }

    Ok(())
}

/// **Stage 2 of 3** — collects the measurement type via inline keyboard.
///
/// Triggered by: callback query while in `WaitingForType`.
///
/// On success: advances state to `WaitingForDate { plant_id, weight, type_ }`
/// and sends the date selection keyboard.
///
/// Ignores unknown callback data silently.
pub async fn receive_type(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (plant_id, weight): (i64, f32),
) -> HandlerResult {
    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;

        if let Some(type_) = parse_measurement_type(&data) {
            dialogue
                .update(MeasurementDialogue::WaitingForDate {
                    plant_id,
                    weight,
                    type_,
                })
                .await?;

            let chat_id = q.message.unwrap().chat().id;
            bot.send_message(chat_id, "Выберите дату: ")
                .reply_markup(date_keyboard())
                .await?;
        }
    }
    Ok(())
}

/// **Stage 3 of 3** — collects the date and writes the measurement to the DB.
///
/// Triggered by: callback query while in `WaitingForDate`.
///
/// On success: writes measurement via [`db_operations::create_measurement`],
/// resets state to `WaitingForPlant`, and shows the plant selection keyboard.
///
/// On failure: replies with a date format error and stays in the current state.
pub async fn receive_date(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    pool: PgPool,
    (plant_id, weight, type_): (i64, f32, MeasurementType),
) -> HandlerResult {
    let chat_id = q.message.unwrap().chat().id;

    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;

        if let Some(date) = parse_date(&data) {
            bot.send_message(
                chat_id,
                format!(
                    "Записано: {} г., тип полива: {:?}, дата: {}",
                    weight, type_, date
                ),
            )
            .await?;

            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id.0).await?;

            db_operations::create_measurement(&pool, plant_id, weight, date, type_.to_string())
                .await?;
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants, "start"))
                .await?;
        } else {
            bot.send_message(chat_id, "Неверный формат даты. Попробуйте еще раз")
                .await?;
        }
    }

    Ok(())
}
