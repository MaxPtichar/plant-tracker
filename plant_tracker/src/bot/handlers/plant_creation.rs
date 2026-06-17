use sqlx::PgPool;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*};

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue, PlantCreationDialogue, dialogue::WateringConfigDialog, keyboards::{ back_to}
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
pub async fn get_plant_name(bot: Bot, dialogue: MyDialogue, msg: Message, pool: PgPool) -> HandlerResult {

    const MAX_PLANT_NAME_LENGTH: usize = 40;

    let plants_name = match msg.text() {
        Some(name) => name.trim(), 
        None => {
            bot.send_message(msg.chat.id, "❌ Пожалуйста, отправьте текстовое название.").await?;
        return Ok(());
        }
    };

    if plants_name.is_empty() {
         bot.send_message(msg.chat.id, "⚠️ Название не может быть пустым. Введите имя растения:").await?;
        return Ok(());
    }

    let name_len = plants_name.chars().count();

    if name_len > MAX_PLANT_NAME_LENGTH {
        bot.send_message(msg.chat.id, format!("⚠️ **Слишком длинное название!**\n\
             Максимальная длина: {} символов (сейчас: {}).\n\n\
             Пожалуйста, придумайте имя покороче:", 
            MAX_PLANT_NAME_LENGTH, name_len
        )).await?;
        return Ok(());
    }
    else if !plants_name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() ) {
        bot.send_message(msg.chat.id, "⚠️ **Ошибка!** Название должно содержать только буквы, цифры и пробелы.\n\
         Попробуйте еще раз:")
         .await?;
        return Ok(());
    }


    else {
        let chat_id = msg.chat.id.0;
          let plant_id = db_operations::create_new_plant(
            &pool,
            chat_id,
            &plants_name,
            )
        .await?;

    tracing::info!("User {} created new plant with name {}", &chat_id, &plants_name);

            bot.send_message(
    msg.chat.id, 
    format!(
        "✅ Растение *{}* успешно добавлено!\n\n\
         ⚖️ Введите вес политого растения в граммах:", 
        &plants_name
    )

    
)
.await?;
dialogue
            .update(MeasurementDialogue::WateringConfig(
                WateringConfigDialog::WaitingWetWeight { plant_id },
            ))
            .await?;

    }


    Ok(())
}
