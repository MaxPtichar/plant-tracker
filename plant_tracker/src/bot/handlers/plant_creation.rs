use sqlx::PgPool;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*, types::MessageId};

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue, PlantCreationDialogue,
        dialogue::WateringConfigDialog, keyboards::back_to,
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
//
// UX note: the dialogue lives on a single "card" message
// (id = prev_msg_id) that gets edited in place at every step.
// The id never changes since we only ever call edit_message_text,
// so prev_msg_id is simply forwarded unchanged between states.
// User-sent text messages are deleted right after being read to
// keep that single-screen feel.
// ============================================================

/// **Stage 1 of 3** — collects the plant's display name.
///
/// Triggered by: text message while in `WaitingForName`.
///
/// On success: advances state to `WaitingForMoisture { name }`
/// (and persists the plant), editing the card to ask for wet weight.
///
/// On failure: edits the card with an error and stays in the current state.
pub async fn get_plant_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
    prev_msg_id: MessageId,
) -> HandlerResult {
    const MAX_PLANT_NAME_LENGTH: usize = 40;

    let chat_id = msg.chat.id;

    // Сообщение пользователя больше не нужно — экран "одного сообщения".
    bot.delete_message(chat_id, msg.id).await.ok();

    let plants_name = match msg.text() {
        Some(name) => name.trim(),
        None => {
            bot.edit_message_text(chat_id, prev_msg_id, "❌ Пожалуйста, отправьте текстовое название.")
                .await?;
            return Ok(());
        }
    };

    if plants_name.is_empty() {
        bot.edit_message_text(
            chat_id,
            prev_msg_id,
            "⚠️ Название не может быть пустым. Введите имя растения:",
        )
        .await?;
        return Ok(());
    }

    let name_len = plants_name.chars().count();

    if name_len > MAX_PLANT_NAME_LENGTH {
        bot.edit_message_text(
            chat_id,
            prev_msg_id,
            format!(
                "⚠️ **Слишком длинное название!**\n\
                 Максимальная длина: {} символов (сейчас: {}).\n\n\
                 Пожалуйста, придумайте имя покороче:",
                MAX_PLANT_NAME_LENGTH, name_len
            ),
        )
        .await?;
        return Ok(());
    }

    if !plants_name
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace())
    {
        bot.edit_message_text(
            chat_id,
            prev_msg_id,
            "⚠️ **Ошибка!** Название должно содержать только буквы, цифры и пробелы.\n\
             Попробуйте еще раз:",
        )
        .await?;
        return Ok(());
    }

    let plant_id = db_operations::create_new_plant(&pool, chat_id.0, plants_name).await?;

    tracing::info!("User {} created new plant with name {}", chat_id.0, plants_name);

    bot.edit_message_text(
        chat_id,
        prev_msg_id,
        format!(
            "✅ Растение *{}* успешно добавлено!\n\n\
             ⚖️ Введите вес политого растения в граммах:",
            plants_name
        ),
    )
    .await?;

    dialogue
        .update(MeasurementDialogue::WateringConfig(
            WateringConfigDialog::WaitingWetWeight { prev_msg_id, plant_id },
        ))
        .await?;

    Ok(())
}