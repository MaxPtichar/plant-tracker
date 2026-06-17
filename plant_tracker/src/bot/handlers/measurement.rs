use std::f32::consts;

use anyhow::Context;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use sqlx::PgPool;
use teloxide::prelude::*;

use crate::analytycs_v2::daily_water_loss;
use crate::bot::dialogue::WateringConfigDialog;
use crate::db_operations::{check_water_config, get_daily_loss, get_last_after_watering};
use crate::models::{MeasurementType, PlantMeasurementsHistory};
use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue,
        callbacks::{parse_date, parse_measurement_type},
        keyboards::{date_keyboard, measurement_type_keyboard, plant_keyboard},
    },
    db_operations,
};

//коэффициент сглаживания
const ALPHA: f32 = 0.02;

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
pub async fn receive_plant(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    if let Some(data) = q.data {
        dbg!(&data);
        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();
        let config = check_water_config(&pool, plant_id).await?;
        let chat_id = q.message.context("ChatID doesn't exists")?.chat().id;
        if !config {
            bot.send_message(
   chat_id,
    format!(
        "⚠️ Настройки полива для этого цветка еще не заданы.\n\
         Чтобы бот мог правильно рассчитывать влажность почвы и присылать напоминания, нам нужно провести быструю калибровку.\n\n\
         ⚖️ Пожалуйста, введите вес ПОЛИТОГО растения в граммах (например: 1250):", 
    )
)
.await?;
            dialogue
                .update(MeasurementDialogue::WateringConfig(
                    WateringConfigDialog::WaitingWetWeight { plant_id },
                ))
                .await?;
            return Ok(());
        }

        bot.answer_callback_query(q.id).await?;

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
    (plant_id, _plant_name): (i64, String),
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

    if let Some(data) = q.data.as_deref() {
        bot.answer_callback_query(q.id).await?;

        if let Some(date) = parse_date(data) {
            return finalize_measurement(
                bot,
                dialogue,
                chat_id,
                pool,
                (plant_id, weight, type_, date),
            )
            .await;
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
    (plant_id, weight, type_, date): (i64, f32, MeasurementType, DateTime<Utc>),
) -> HandlerResult {
    db_operations::create_measurement(&pool, plant_id, weight, date, type_.to_string()).await?;
    tracing::info!(
        "Created new measurement: plant_id: {}, date: {}",
        plant_id,
        date
    );
    let date_str = date.format("%d.%m.%Y").to_string();

    let message = format!(
        "🌱 **Запись зафиксирована**\n\
         📅 Дата: {}\n\
         ⚖️ Вес растения: {} г.\n\
         💧 Режим: {}",
        date_str, weight, type_
    );

    bot.send_message(chat_id, message).await?;

    //update learned_daily_loss if measurement type is regular

    let mut pl_detail: Vec<PlantMeasurementsHistory> = Vec::with_capacity(2);

    if type_ == MeasurementType::Regular {
        if let Ok(Some((watering_weight, watering_date))) =
            get_last_after_watering(&pool, plant_id).await
        {
            let current_regular = PlantMeasurementsHistory {
                weight,
                date,
                measuring_type: type_.to_string(),
            };
            let last_after_watering = PlantMeasurementsHistory {
                weight: watering_weight,
                date: watering_date,
                measuring_type: String::from("AfterWatering"),
            };

            pl_detail.push(current_regular);
            pl_detail.push(last_after_watering);

            if let Some(loss) = daily_water_loss(&pl_detail) {
                let old_loss_opt = get_daily_loss(&pool, plant_id).await?;

                let target_emal = match old_loss_opt {
                    Some(l) => ALPHA * loss + (1.0 - ALPHA) * l,
                    None => loss,
                };

                tracing::info!("plant_id: {}, daily_loss: {}: ", plant_id, target_emal);
                db_operations::update_daily_loss(&pool, plant_id, target_emal).await?;
                tracing::info!("Daily loss is updated.")
            }
        }

        
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
    if let Some(text) = msg.text()
        && let Ok(date) = NaiveDate::parse_from_str(text, "%d.%m.%Y")
            .or_else(|_| NaiveDate::parse_from_str(text, "%d.%m.%y"))
    {
        let date = date.and_hms_opt(0, 0, 0).unwrap();
        let datetime_utc: DateTime<Utc> = Utc.from_local_datetime(&date).unwrap();

        return finalize_measurement(
            bot,
            dialogue,
            msg.chat.id,
            pool,
            (plant_id, weight, type_, datetime_utc),
        )
        .await;
    }

    bot.send_message(
        msg.chat.id,
        "Неверный формат. Нужно ДД.ММ.ГГГГ или ДД.ММ.ГГ (например 06.04.2026 или 06.04.26):",
    )
    .await?;
    Ok(())
}
