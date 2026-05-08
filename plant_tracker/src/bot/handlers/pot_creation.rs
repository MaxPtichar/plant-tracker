use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue,
        keyboards::{back_to, main_menu_buttons, soil_type_keyboard},
    },
    db_operations,
};

use crate::bot::dialogue::PotCreationDialog;

/// Handles plant selection for pot configuration.
///
/// Parses `plant_id` and `plant_name` from callback data (`"id:name"`),
/// asks the user to enter the empty pot weight in grams,
/// and transitions to [`PotCreationDialog::WaitingForPotWeight`].
pub async fn receive_plant_for_pot(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
) -> HandlerResult {
    if let Some(data) = q.data {
        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();

        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(
            chat_id,
            format!("☘️ {plant_name}\n\nВведите вес пустого горшка в граммах:"),
        )
        .reply_markup(back_to())
        .await?;

        dialogue
            .update(MeasurementDialogue::CreatingPot(
                PotCreationDialog::WaitingForPotWeight { plant_id },
            ))
            .await?;
    }
    Ok(())
}

/// Handles empty pot weight input.
///
/// Expects a text message with an integer weight in grams.
/// On success transitions to [`PotCreationDialog::WaitingForDrySoilWeight`].
/// On invalid input asks the user to retry.
///
/// Called when the dialogue is in [`PotCreationDialog::WaitingForPotWeight`].
pub async fn recieve_pot_weight(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    plant_id: i64,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(pot_weight) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "\n\nМне нужно знать «нулевую точку» (вес без воды).
Введите общий вес горшка с сухой землей в граммах:»"
                    ),
                )
                .await?;

                dialogue
                    .update(MeasurementDialogue::CreatingPot(
                        PotCreationDialog::WaitingForDrySoilWeight {
                            plant_id,
                            pot_weight,
                        },
                    ))
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
    };

    Ok(())
}

/// Handles dry soil weight input and saves the pot configuration.
///
/// Expects a text message with an integer weight in grams.
/// On success:
/// - saves the pot config via [`db_operations::create_pot_config`]
///   (deactivates previous config automatically)
/// - shows a summary with pot weight, dry soil weight and total dry weight
/// - exits the dialogue
///
/// Called when the dialogue is in [`PotCreationDialog::WaitingForDrySoilWeight`].
pub async fn receive_dry_soil_weight(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (plant_id, pot_weight): (i64, i64),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(dry_soil_weight) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "🪴 Отлично! Почти готово.

Измерьте диаметр горшка по верхнему краю и введите в сантиметрах.

Например: 12, 16, 20",
                    ),
                )
                .await?;

                dialogue
                    .update(MeasurementDialogue::CreatingPot(
                        PotCreationDialog::WaitingForDiameter {
                            plant_id,
                            pot_weight,
                            dry_soil_weight,
                        },
                    ))
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

pub async fn receive_pot_diameter(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (plant_id, pot_weight, dry_soil_weight): (i64, i64, i64),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<f32>() {
            Ok(pot_diameter_cm) => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "🌱 Какой грунт используется?

Тип грунта влияет на то, как быстро земля отдаёт воду корням.
Если не уверены — выбирайте универсальный.",
                    ),
                )
                .reply_markup(soil_type_keyboard())
                .await?;

                dialogue
                    .update(MeasurementDialogue::CreatingPot(
                        PotCreationDialog::WaitingForSoilType {
                            plant_id,
                            pot_weight,
                            dry_soil_weight,
                            pot_diameter_cm,
                        },
                    ))
                    .await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Введите число").await?;
            }
        },

        None => {
            bot.send_message(msg.chat.id, "Введите диаметр в сантиметрах")
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_soil_type(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    pool: PgPool,
    (plant_id, pot_weight, dry_soil_weight, pot_diameter_cm): (i64, i64, i64, f32),
) -> HandlerResult {
    bot.answer_callback_query(q.id).await?;

    let soil_type = match q.data.as_deref() {
        Some("tropical") => "tropical",
        Some("succulent") => "succulent",
        _ => "universal",
    };

    let chat_id = q.message.unwrap().chat().id;

    db_operations::create_pot_config(
        &pool,
        plant_id,
        pot_weight,
        dry_soil_weight,
        pot_diameter_cm,
        soil_type,
    )
    .await?;

    bot.send_message(
        chat_id,
        format!(
            "✅ Горшок настроен!\n\n\
         🪴 ID растения: {}\n\
         ⚖️ Вес пустого горшка: {} г.\n\
         ⏳ Вес сухой почвы: {} г.\n\
         💧 Общий базовый вес (сухой): {} г.\n\
         📐 Диаметр горшка: {} см.\n\
         🌱 Тип почвы: {}\n\n\
         Теперь при взвешивании я смогу точно рассчитать остаток влаги.",
            plant_id,
            pot_weight,
            dry_soil_weight,
            pot_weight + dry_soil_weight,
            pot_diameter_cm,
            soil_type
        ),
    )
    .reply_markup(main_menu_buttons())
    .await?;

    dialogue.exit().await?;

    Ok(())
}
