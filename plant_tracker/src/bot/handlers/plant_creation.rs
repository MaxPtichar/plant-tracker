use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue, PlantCreationDialogue,
        dialogue::PotCreationDialog,
        keyboards::{
            air_circulation_keyboard, get_type_of_moisture, light_level_keyboard,
            main_menu_buttons, plant_type_keyboard,
        },
    },
    db_operations,
};

// ============================================================
// Plant creation — multi-step dialogue
//
// Drives a three-state FSM that collects a plant name and its
// target residual-moisture threshold, then persists the result.
//
// States (PlantCreationDialogue):
//   WaitingForName          → expects a text message
//   WaitingForMoisture      → expects an inline-keyboard callback
//   WaitingForCustomMoisture → expects a text message (float)
//
// Handler routing (to be wired in the dispatcher):
//   text msg  + WaitingForName           → get_plant_name
//   callback  + WaitingForMoisture       → get_moisture
//   text msg  + WaitingForCustomMoisture → get_custom_moisture
// ============================================================

/// **Stage 1 of 3** — collects the plant's display name.
///
/// Triggered by: text message while in `WaitingForName`.
///
/// On success: advances state to `WaitingForMoisture { name }`
/// and sends the moisture keyboard ([`get_type_of_moisture`]).
///
/// On failure: replies with an error and stays in the current state.
pub async fn get_plant_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(plants_name) => {
            let name = plants_name.to_string();

            dialogue
                .update(MeasurementDialogue::CreatingPlant(
                    PlantCreationDialogue::WaitingForPlantType { name },
                ))
                .await?;
            bot.send_message(msg.chat.id, "Выберите тип растения:")
                .reply_markup(plant_type_keyboard())
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Растение должно иметь название!")
                .await?;
        }
    }

    Ok(())
}

pub async fn get_plant_type(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    name: String,
) -> HandlerResult {
    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;

        let plant_type = match data.as_str() {
            "Succulent" => "Succulent".to_string(),
            "Tropical" => "Tropical".to_string(),
            _ => "Regular".to_string(),
        };

        dialogue
            .update(MeasurementDialogue::CreatingPlant(
                PlantCreationDialogue::WaitingForLightLevel { name, plant_type },
            ))
            .await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(chat_id, "Где стоит растение?🌿")
            .reply_markup(light_level_keyboard())
            .await?;
    }
    Ok(())
}

pub async fn get_light_level(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (name, plant_type): (String, String),
) -> HandlerResult {
    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;

        let light_level = match data.as_str() {
            "window" => "window".to_string(),
            _ => "shadow".to_string(),
        };

        dialogue
            .update(MeasurementDialogue::CreatingPlant(
                PlantCreationDialogue::WaitingForAirCirculation {
                    name,
                    plant_type,
                    light_level,
                },
            ))
            .await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(
            chat_id,
            "Есть ли рядом с растением движение воздуха?\n\n
🌬️ Сквозняк, открытое окно, вентилятор\n
😶 Тихое место, воздух не движется",
        )
        .reply_markup(air_circulation_keyboard())
        .await?;
    }
    Ok(())
}

pub async fn get_air_circ(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    pool: PgPool,
    (name, plant_type, light_level): (String, String, String),
) -> HandlerResult {
    if let Some(data) = q.data {
        dbg!(&data);

        bot.answer_callback_query(q.id).await?;
        dbg!("answered callback");

        let air_circulation = match data.as_str() {
            "normal" => "normal".to_string(),
            _ => "stagnant".to_string(),
        };

        dbg!(&air_circulation);

        let chat_id = q.message.unwrap().chat().id;
        dbg!(&chat_id);

        let plant_id = db_operations::create_new_plant(
            &pool,
            chat_id.0,
            &name,
            &plant_type,
            &light_level,
            &air_circulation,
        )
        .await?;

        dbg!(plant_id);

        dialogue
            .update(MeasurementDialogue::CreatingPot(
                PotCreationDialog::WaitingForPotWeight { plant_id },
            ))
            .await?;

        bot.send_message(
            chat_id,
            format!(
                "🌱 Растение добавлено!\n\n\
             📛 Название: {}\n\
             🌿 Тип: {}\n\
             ☀️ Освещение: {}\n\
             💨 Циркуляция воздуха: {}\n\n\
             Теперь настроим горшок.\n\
             Введите вес пустого горшка в граммах:",
                name,
                match plant_type.as_str() {
                    "Tropical" => "Тропическое",
                    "Succulent" => "Суккулент",
                    _ => "Обычное",
                },
                match light_level.as_str() {
                    "window" => "У окна ☀️",
                    _ => "В тени 🌥️",
                },
                match air_circulation.as_str() {
                    "normal" => "Есть движение воздуха 🌬️",
                    _ => "Воздух не движется 😶",
                },
            ),
        )
        .await?;
    }
    Ok(())
}
