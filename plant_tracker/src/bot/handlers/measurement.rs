use chrono::NaiveDate;
use sqlx::{PgPool, pool};
use teloxide::prelude::*;

use crate::models::MeasurementType;
use crate::operations::update_avg_cycle;
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
            format!("☘️ {plant_name}\n\nВведите текущий вес растения в граммах:"),
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
    pool: PgPool,
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

    if let Some(data) = q.data.as_deref() {
        bot.answer_callback_query(q.id).await?;

        if let Some(date) = parse_date(&data) {
            return Ok(finalize_measurement(
                bot,
                dialogue,
                chat_id,
                pool,
                (plant_id, weight, type_, date),
            )
            .await?);
        }
        dialogue
            .update(MeasurementDialogue::WaitingForCustomDate {
                plant_id,
                weight,
                type_,
            })
            .await?;

        bot.send_message(chat_id, "Введите дату в формате ДД.ММ.ГГГГ:")
            .await?;
    }

    Ok(())
}

pub async fn finalize_measurement(
    bot: Bot,
    dialogue: MyDialogue,
    chat_id: ChatId,
    pool: PgPool,
    (plant_id, weight, type_, date): (i64, f32, MeasurementType, NaiveDate),
) -> HandlerResult {
    db_operations::create_measurement(&pool, plant_id, weight, date, type_.to_string()).await?;
    bot.send_message(
        chat_id,
        format!(
            "Записано: {} г., тип полива: {:?}, дата: {}",
            weight, type_, date
        ),
    )
    .await?;
    if type_ == MeasurementType::Regular {
        let user_id = chat_id.0 as i64;
        update_avg_cycle(plant_id, &pool, user_id).await?;
    }

    dialogue
        .update(MeasurementDialogue::WaitingForPlant)
        .await?;

    let plants: Vec<crate::models::Plant> =
        db_operations::get_user_plants(&pool, chat_id.0).await?;

    bot.send_message(chat_id, "Выбери растение: ")
        .reply_markup(plant_keyboard(&plants, "Start"))
        .await?;

    Ok(())
}

pub async fn receive_custom_date(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    pool: PgPool,
    (plant_id, weight, type_): (i64, f32, MeasurementType),
) -> HandlerResult {
    if let Some(text) = msg.text() {
        if let Ok(date) = NaiveDate::parse_from_str(text, "%d.%m.%Y")
            .or_else(|_| NaiveDate::parse_from_str(text, "%d.%m.%y"))
        {
            return Ok(finalize_measurement(
                bot,
                dialogue,
                msg.chat.id,
                pool,
                (plant_id, weight, type_, date),
            )
            .await?);
        }
    }

    bot.send_message(
        msg.chat.id,
        "Неверный формат. Нужно ДД.ММ.ГГГГ или ДД.ММ.ГГ (например 06.04.2026 или 06.04.26):",
    )
    .await?;
    Ok(())
}
