use std::ops::Div;

use anyhow::Context;
use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue,
        dialogue::WateringConfigDialog,
        keyboards::{back_to, back_to_my_plants, main_menu_buttons},
    },
    db_operations,
};

/// Handles plant selection for pot configuration.
///
/// Parses `plant_id` and `plant_name` from callback data (`"id:name"`),
/// asks the user to enter the empty pot weight in grams,
/// and transitions to [`PotCreationDialog::WaitingForPotWeight`].
pub async fn receive_plant_for_config(
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
            format!(
                "☘️ {plant_name}\n\n\
         Введите вес политого растения в граммах.\n\n\
         Зачем это нужно: бот запомнит максимальный вес горшка сразу после полива. \
         Сравнивая его с последующими взвешиваниями, он поймет, как быстро испаряется вода, \
         и вовремя напомнит вам, когда земля станет сухой."
            ),
        )
        .reply_markup(back_to_my_plants())
        .await?;

        dialogue
            .update(MeasurementDialogue::WateringConfig(
                WateringConfigDialog::WaitingWetWeight { plant_id },
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
pub async fn recieve_wet_weight(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    plant_id: i64,
) -> HandlerResult {
    const MAX_WEIGHT: i64 = 50_000;

    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(wet_weight) if wet_weight > 0 && wet_weight <= MAX_WEIGHT => {
                bot.send_message(
                    msg.chat.id,
                    format!(
        "
         Введите вес полностью сухой земли в граммах.\n\n\
         Зачем это нужно: это нижняя граница веса, при которой растению уже критически не хватает влаги. \
         Зная вес сытого растения и вес сухого горшка, бот сможет определять влажность почвы в процентах \
         при каждом новом замере."
    )
                )
                .await?;

                dialogue
                    .update(MeasurementDialogue::WateringConfig(
                        WateringConfigDialog::WaitingForDrySoilWeight {
                            plant_id,
                            wet_weight,
                        },
                    ))
                    .await?;
            }

            Ok(_) => {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ Вес должен быть числом от 1 до 50000 грамм. Попробуйте еще раз:",
                )
                .await?;
                return Ok(());
            }

            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ Пожалуйста, введите вес цифрами (например: 450). Попробуйте еще раз:",
                )
                .await?;
                return Ok(());
            }
        },

        None => {
            bot.send_message(msg.chat.id, "Отправьте вес растения числом в граммах:")
                .await?;
            return Ok(());
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
    (plant_id, wet_weight): (i64, i64),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(dry_weight) if dry_weight > 0 && dry_weight < wet_weight => {
                bot.send_message(
                    msg.chat.id,
                    "
Укажите, при каком проценте испарившейся воды нужно напомнить о поливе (например, 67).\n\n\
Как это работает: 0% — горшок только что полили, вся вода на месте. 100% — вся доступная вода полностью испарилась.\n\n\
Полезная подсказка: большинство сайтов пишут \"полить когда осталось 30–40% влаги\" — \
это значит в наш бот нужно ввести 60–70 (сколько УЖЕ испарилось).\n\n\
Для суккулентов и сансеверии — 80–90.\n\
Для фикусов и обычных растений — 60–75.\n\
Для влаголюбивых (хлорофитум) — 50–60."
                )
                .await?;

                dialogue
                    .update(MeasurementDialogue::WateringConfig(
                        WateringConfigDialog::WaitingForThresholdPct {
                            plant_id,
                            wet_weight,
                            dry_weight,
                        },
                    ))
                    .await?;
            }

            Ok(_) => {
                bot.send_message(
            msg.chat.id,
            "⚠️ Некорректный вес! Число должно быть больше 0 грамм и меньше веса политого растения.\n\n\
             Пожалуйста, введите вес сухой земли еще раз:"
        )
        .await?;
                return Ok(());
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "⚠️ Пожалуйста, введите вес сухой земли только цифрами (без букв и знаков, например: 350):").await?;
                return Ok(());
            }
        },

        None => {
            bot.send_message(msg.chat.id, "⚠️ Пожалуйста, введите вес в граммах: ")
                .await?;
            return Ok(());
        }
    }

    Ok(())
}

pub async fn receive_threshold_pct(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
    (plant_id, wet_weight, dry_weight): (i64, i64, i64),
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(threshold_pct) if threshold_pct > 0 && threshold_pct < 100 => {
                bot.send_message(msg.chat.id, format!(
        "🎉 Полив успешно настроен!\n\n\
         📋 Итоговые настройки:\n\
         • Вес после полива: {} г\n\
         • Вес сухой земли: {} г\n\
         • Напомнить о поливе, когда испарится: {}%\n\n\
         Бот принял данные. Теперь при каждом взвешивании я буду показывать текущую влажность и подскажу, когда пора доставать лейку! 🌿",
        wet_weight, dry_weight, threshold_pct
    ))
                    .reply_markup(back_to())
                    .await?;

                db_operations::create_watering_config(
                    &pool,
                    plant_id,
                    wet_weight,
                    dry_weight,
                    (threshold_pct as f32).div(100.0),
                )
                .await?;

                tracing::info!("User {} created new water config.", msg.chat.id.0);

                dialogue.exit().await?;
            }

            Ok(_) => {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ Ошибка! Процент должен быть целым числом в диапазоне от 1 до 99.\n\n\
                     Пожалуйста, введите корректное число:",
                )
                .await?;
                return Ok(());
            }

            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "⚠️ Пожалуйста, введите порог полива только цифрами, без знака % (например: 65).\n\n\
                     Попробуйте еще раз:"
                )
                .await?;
                return Ok(());
            }
        },

        None => {
            bot.send_message(
                msg.chat.id,
                "⚠️ Сообщение не содержит текста. Отправьте число от 1 до 99:",
            )
            .await?;
            return Ok(());
        }
    }

    Ok(())
}

///Для валидации
pub async fn ensure_water_config(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: &MyDialogue,
    pool: &sqlx::PgPool,
    plant_id: i64,
) -> anyhow::Result<bool> {
    // 1. Делаем запрос в БД с контекстом на случай ошибки связи с базой
    let has_config = crate::db_operations::check_water_config(pool, plant_id)
        .await
        .context("Не удалось проверить наличие water_config в базе данных")?;

    // 2. Если конфигурация отсутствует
    if !has_config {
        // Отправляем пользователю понятный текст о калибровке
        bot.send_message(
            chat_id,
            "⚠️ Настройки полива для этого цветка еще не заданы.\n\
             Чтобы бот мог правильно рассчитывать влажность почвы и присылать напоминания, нам нужно провести быструю калибровку.\n\n\
             ⚖️ Пожалуйста, введите вес ПОЛИТОГО растения в граммах (например: 1250):",
        )
        .await
        .context("Ошибка отправки сообщения о начале калибровки")?;

        // Переводим диалог на шаг ожидания мокрого веса
        dialogue
            .update(MeasurementDialogue::WateringConfig(
                WateringConfigDialog::WaitingWetWeight { plant_id },
            ))
            .await
            .context("Не удалось обновить стейт диалога на WaitingWetWeight")?;

        return Ok(false);
    }

    // Если всё настроено, возвращаем true
    Ok(true)
}
