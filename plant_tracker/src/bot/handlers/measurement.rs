use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use crate::analytycs_v2::daily_water_loss;
use crate::bot::dialogue::WateringConfigDialog;
use crate::bot::keyboards::plant_or_menu;

use crate::models::{MeasurementType, PlantMeasurementsHistory};
use crate::prelude::*;
use crate::{
    bot::{
        callbacks::{parse_date, parse_measurement_type},
        keyboards::{date_keyboard, measurement_type_keyboard},
    },
    db_operations,
};

const ALPHA: f32 = 0.02;

pub async fn receive_plant(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    let Some(data) = q.data else { return Ok(()) };
    let Some(message) = q.message.as_ref() else {
        return Ok(());
    };
    let chat_id = message.chat().id;
    let msg_id = message.id();

    let Some((plant_id, plant_name)) = data
        .split_once(':')
        .and_then(|(id, name)| id.parse::<i64>().ok().map(|id| (id, name.to_string())))
    else {
        return Ok(());
    };

    bot.answer_callback_query(q.id).await?;

    let config = db_operations::check_water_config(&pool, plant_id).await?;

    if !config {
        bot.edit_message_text(
            chat_id,
            msg_id,
            "⚠️ Настройки полива для этого цветка еще не заданы.\n\
             Чтобы бот мог правильно рассчитывать влажность почвы и присылать напоминания, нам нужно провести быструю калибровку.\n\n\
             ⚖️ Пожалуйста, введите вес ПОЛИТОГО растения в граммах (например: 1250):",
        )
        .await?;

        dialogue
            .update(MeasurementDialogue::WateringConfig(
                WateringConfigDialog::WaitingWetWeight {
                    prev_msg_id: msg_id,
                    plant_id,
                },
            ))
            .await?;

        return Ok(());
    }

    bot.edit_message_text(
        chat_id,
        msg_id,
        format!("☘️ {plant_name}\n\nВведите текущий вес растения в граммах:"),
    )
    .await?;

    dialogue
        .update(MeasurementDialogue::WaitingForWeight {
            prev_msg_id: msg_id,
            plant_id,
        })
        .await?;

    Ok(())
}

pub async fn receive_weight(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (prev_msg_id, plant_id): (MessageId, i64),
) -> HandlerResult {
    bot.delete_message(msg.chat.id, msg.id).await.ok();

    match msg.text() {
        Some(text) => match text.parse::<f32>() {
            Ok(weight) => {
                dialogue
                    .update(MeasurementDialogue::WaitingForType {
                        prev_msg_id,
                        plant_id,
                        weight,
                    })
                    .await?;

                bot.edit_message_text(msg.chat.id, prev_msg_id, "Выбери тип измерений: ")
                    .reply_markup(measurement_type_keyboard())
                    .await?;
            }
            Err(_) => {
                bot.edit_message_text(msg.chat.id, prev_msg_id, "Введите число")
                    .await?;
            }
        },
        None => {
            bot.edit_message_text(msg.chat.id, prev_msg_id, "Введите вес в граммах")
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_type(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (prev_msg_id, plant_id, weight): (MessageId, i64, f32),
) -> HandlerResult {
    let Some(data) = q.data else { return Ok(()) };
    bot.answer_callback_query(q.id).await?;

    let Some(message) = q.message.as_ref() else {
        return Ok(());
    };
    let chat_id = message.chat().id;

    if let Some(type_) = parse_measurement_type(&data) {
        dialogue
            .update(MeasurementDialogue::WaitingForDate {
                prev_msg_id,
                plant_id,
                weight,
                type_,
            })
            .await?;

        bot.edit_message_text(chat_id, prev_msg_id, "Выберите дату: ")
            .reply_markup(date_keyboard())
            .await?;
    }

    Ok(())
}

pub async fn receive_date(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    pool: PgPool,
    (prev_msg_id, plant_id, weight, type_): (MessageId, i64, f32, MeasurementType),
) -> HandlerResult {
    let Some(message) = q.message.as_ref() else {
        return Ok(());
    };
    let chat_id = message.chat().id;
    let Some(data) = q.data.as_deref() else {
        return Ok(());
    };

    bot.answer_callback_query(q.id).await?;

    let date = match parse_date(data) {
        Some(date) => date,
        None => parse_custom_date(data)?,
    };

    return finalize_measurement(
        bot,
        dialogue,
        chat_id,
        prev_msg_id,
        pool,
        (plant_id, weight, type_, date),
    )
    .await;
}

pub async fn finalize_measurement(
    bot: Bot,
    dialogue: MyDialogue,
    chat_id: ChatId,
    prev_msg_id: MessageId,
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

    let result_text = format!(
        "🌱 **Запись зафиксирована**\n\
         📅 Дата: {}\n\
         ⚖️ Вес растения: {} г.\n\
         💧 Режим: {}",
        date_str, weight, type_
    );

    let mut pl_detail: Vec<PlantMeasurementsHistory> = Vec::with_capacity(2);

    if type_ == MeasurementType::Regular
        && let Ok(Some((watering_weight, watering_date))) =
            db_operations::get_last_after_watering(&pool, plant_id).await
    {
        pl_detail.push(PlantMeasurementsHistory {
            weight,
            date,
            measuring_type: type_.to_string(),
        });
        pl_detail.push(PlantMeasurementsHistory {
            weight: watering_weight,
            date: watering_date,
            measuring_type: String::from("AfterWatering"),
        });

        if let Some(loss) = daily_water_loss(&pl_detail) {
            let old_loss_opt = db_operations::get_daily_loss(&pool, plant_id).await?;
            let target_emal = match old_loss_opt {
                Some(l) => ALPHA * loss + (1.0 - ALPHA) * l,
                None => loss,
            };

            tracing::info!("plant_id: {}, daily_loss: {}: ", plant_id, target_emal);
            db_operations::update_daily_loss(&pool, plant_id, target_emal).await?;
            tracing::info!("Daily loss is updated.")
        }
    }

    dialogue
        .update(MeasurementDialogue::WaitingForPlant)
        .await?;

    bot.edit_message_text(chat_id, prev_msg_id, format!("{result_text}\n\n"))
        .reply_markup(plant_or_menu())
        .await?;

    Ok(())
}

fn parse_custom_date(data: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let naive = NaiveDate::parse_from_str(data, "%d.%m.%Y")?;
    let naive_dt = naive.and_hms_opt(0, 0, 0).unwrap();
    Ok(Utc.from_local_datetime(&naive_dt).unwrap())
}
