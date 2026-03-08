

use teloxide::prelude::*;

use crate::add_new_measurement::add_new_measurement;
use crate::bot::callbacks::{
     parse_date, parse_measurement_type,
};
use crate::bot::keyboards::{back_to, date_keyboard, measurement_type_keyboard, plant_keyboard};
use crate::bot::{HandlerResult, MyDialogue};
use crate::models::MeasurementType;
use crate::storage::load;


#[derive(Clone, Default)]
pub enum MeasurementDialogue {
    #[default]
    WaitingForPlant,
    WaitingForWeight {
        plant_id: u32,
    },
    WaitingForType {
        plant_id: u32,
        weight: f32,
    },
    WaitingForDate {
        plant_id: u32,
        weight: f32,
        type_: MeasurementType,
    },
}

//get weight
pub async fn receive_weight(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    plant_id: u32,
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

pub async fn receive_type(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (plant_id, weight): (u32, f32),
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

pub async fn recieve_date(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (plant_id, weight, type_): (u32, f32, MeasurementType),
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

            let mut plants = load();
            add_new_measurement(&mut plants, plant_id, weight, date, type_);

            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
        } else {
            bot.send_message(chat_id, "Неверный формат даты. Попробуйте еще раз")
                .await?;
        }
    }

    Ok(())
}

// get id plant
pub async fn receive_plant(bot: Bot, q: CallbackQuery, dialogue: MyDialogue) -> HandlerResult {
    if let Some(data) = q.data {
        let plant_id: u32 = data.parse().unwrap();
        bot.answer_callback_query(q.id).await?;

        dialogue
            .update(MeasurementDialogue::WaitingForWeight { plant_id })
            .await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(chat_id, "Введите вес в граммах")
            .reply_markup(back_to())
            .await?;
    }
    Ok(())
}
