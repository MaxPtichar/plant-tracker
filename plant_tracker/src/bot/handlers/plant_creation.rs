use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue, PlantCreationDialogue,
        dialogue::PotCreationDialog,
        keyboards::{get_type_of_moisture, main_menu_buttons},
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
                    PlantCreationDialogue::WaitingForMoisture { name },
                ))
                .await?;
            bot.send_message(msg.chat.id, "Введите остаточный % влаги в горшке")
                .reply_markup(get_type_of_moisture())
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Растение должно иметь название!")
                .await?;
        }
    }

    Ok(())
}

/// **Stage 2 of 3** — reads a preset moisture value from the inline keyboard.
///
/// Triggered by: callback query while in `WaitingForMoisture`.
///
/// Two paths:
/// - `"custom"` data  → advances to `WaitingForCustomMoisture` and asks for
///   a free-form float.
/// - any other value  → parses as `f32`, writes the plant to the DB, shows
///   the main menu, and exits the dialogue.
///
/// Always answers the callback query at the end to dismiss the spinner.
pub async fn get_moisture(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    pool: PgPool,
    name: String,
) -> HandlerResult {
    let chat_id = q.message.unwrap().chat().id;
    if let Some(data) = q.data {
        if data == "custom" {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(
                    PlantCreationDialogue::WaitingForCustomMoisture { name },
                ))
                .await?;
            bot.send_message(chat_id, "Введите число от 0.1 до 1.0 (например, 0.25):")
                .await?;
        } else {
            let moisture: f32 = data.parse().unwrap_or(0.3);

            let plant_id =
                db_operations::create_new_plant(&pool, chat_id.0, &name, moisture).await?;

            bot.send_message(chat_id, format!("🌿 Растение '{name}' добавлено!"))
                .reply_markup(main_menu_buttons())
                .await?;
            bot.send_message(chat_id, "✅ Сохранено!").await?;
            dialogue
                .update(MeasurementDialogue::CreatingPot(
                    PotCreationDialog::WaitingForPotWeight { plant_id },
                ))
                .await?;
            bot.send_message(
                chat_id,
                "🪴 Теперь настроим горшок!\n\nВведите вес пустого горшка в граммах:",
            )
            .await?;
        }
    }
    bot.answer_callback_query(q.id).await?;
    Ok(())
}

/// **Stage 3 of 3 (optional)** — collects a custom moisture value as text.
///
/// Triggered by: text message while in `WaitingForCustomMoisture`.
///
/// Accepts a decimal fraction in `[0.0, 1.0]` (commas normalised to dots).
/// On a valid value: writes the plant to the DB, shows the main menu,
/// and exits the dialogue.
/// On an invalid value: replies with guidance and stays in the current state.
pub async fn get_custom_moisture(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
    name: String,
) -> HandlerResult {
    if let Some(text) = msg.text() {
        if let Ok(val) = text.replace(",", ".").parse::<f32>() {
            if (0.0..=1.0).contains(&val) {
                let plant_id =
                    db_operations::create_new_plant(&pool, msg.chat.id.0, &name, val).await?;
                bot.send_message(msg.chat.id, "✅ Сохранено!").await?;
                dialogue
                    .update(MeasurementDialogue::CreatingPot(
                        PotCreationDialog::WaitingForPotWeight { plant_id },
                    ))
                    .await?;
                bot.send_message(
                    msg.chat.id,
                    "🪴 Теперь настроим горшок!\n\nВведите вес пустого горшка в граммах:",
                )
                .await?;
            } else {
                bot.send_message(msg.chat.id, "Введите число от 0 до 1.")
                    .await?;
            }
        }
    } else {
        bot.send_message(msg.chat.id, "Попробуйте ввести число.")
            .await?;
    }

    Ok(())
}
