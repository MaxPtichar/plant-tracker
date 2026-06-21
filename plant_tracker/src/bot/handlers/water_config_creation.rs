use std::ops::Div;

use crate::{
    bot::{
        dialogue::WateringConfigDialog,
        keyboards::{back_to, back_to_my_plants},
    },
    db_operations,
    prelude::*,
};

/// Handles plant selection for pot configuration.
pub async fn receive_plant_for_config(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    prev_msg_id: MessageId,
) -> HandlerResult {
    let message = q.message.unwrap();
    let chat_id = message.chat().id;
    let msg_id = message.id();

    let Some(data) = q.data else {
        return Ok(());
    };

    let (plant_id, plant_name) = data
        .split_once(':')
        .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
        .unwrap();

    bot.answer_callback_query(q.id).await?;

    bot.edit_message_text(
        chat_id,
        prev_msg_id,
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
            WateringConfigDialog::WaitingWetWeight {
                prev_msg_id: msg_id,
                plant_id,
            },
        ))
        .await?;

    Ok(())
}

/// Handles wet weight input.
pub async fn receive_wet_weight(
    bot: Bot,
    msg: Message,
    dialogue: MyDialogue,
    (prev_msg_id, plant_id): (MessageId, i64),
) -> HandlerResult {
    const MAX_WEIGHT: i64 = 50_000;
    let chat_id = msg.chat.id;
    let msg_id = msg.id;

    bot.delete_message(chat_id, msg_id).await.ok();

    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(wet_weight) if wet_weight > 0 && wet_weight <= MAX_WEIGHT => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "Введите вес полностью сухой земли в граммах.\n\n\
                     Зачем это нужно: это нижняя граница веса, при которой растению уже критически не хватает влаги. \
                     Зная вес сытого растения и вес сухого горшка, бот сможет определять влажность почвы в процентах \
                     при каждом новом замере."
                )
                .await?;

                dialogue
                    .update(MeasurementDialogue::WateringConfig(
                        WateringConfigDialog::WaitingForDrySoilWeight {
                            prev_msg_id,
                            plant_id,
                            wet_weight,
                        },
                    ))
                    .await?;
            }

            Ok(_) => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "⚠️ Вес должен быть числом от 1 до 50000 грамм. Попробуйте еще раз:",
                )
                .await?;
            }

            Err(_) => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "⚠️ Пожалуйста, введите вес цифрами (например: 450). Попробуйте еще раз:",
                )
                .await?;
            }
        },

        None => {
            bot.edit_message_text(
                chat_id,
                prev_msg_id,
                "Отправьте вес растения числом в граммах:",
            )
            .await?;
        }
    };

    Ok(())
}

/// Handles dry soil weight input.
pub async fn receive_dry_soil_weight(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (prev_msg_id, plant_id, wet_weight): (MessageId, i64, i64),
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let msg_id = msg.id;

    bot.delete_message(chat_id, msg_id).await.ok();

    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(dry_weight) if dry_weight > 0 && dry_weight < wet_weight => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "Укажите, при каком проценте испарившейся воды нужно напомнить о поливе (например, 67).\n\n\
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
                            prev_msg_id,
                            plant_id,
                            wet_weight,
                            dry_weight,
                        },
                    ))
                    .await?;
            }

            Ok(_) => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "⚠️ Некорректный вес! Число должно быть больше 0 грамм и меньше веса политого растения.\n\n\
                     Пожалуйста, введите вес сухой земли еще раз:"
                )
                .await?;
            }

            Err(_) => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "⚠️ Пожалуйста, введите вес сухой земли только цифрами (без букв и знаков, например: 350):",
                )
                .await?;
            }
        },

        None => {
            bot.edit_message_text(
                chat_id,
                prev_msg_id,
                "⚠️ Пожалуйста, введите вес в граммах: ",
            )
            .await?;
        }
    }

    Ok(())
}

/// Handles threshold pct input and saves the config.
pub async fn receive_threshold_pct(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
    (prev_msg_id, plant_id, wet_weight, dry_weight): (MessageId, i64, i64, i64),
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let msg_id = msg.id;

    bot.delete_message(chat_id, msg_id).await.ok();

    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(threshold_pct) if threshold_pct > 0 && threshold_pct < 100 => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    format!(
                        "🎉 Полив успешно настроен!\n\n\
                         📋 Итоговые настройки:\n\
                         • Вес после полива: {} г\n\
                         • Вес сухой земли: {} г\n\
                         • Напомнить о поливе, когда испарится: {}%\n\n\
                         Бот принял данные. Теперь при каждом взвешивании я буду показывать текущую влажность и подскажу, когда пора доставать лейку! 🌿",
                        wet_weight, dry_weight, threshold_pct
                    ),
                )
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

                tracing::info!("User {} created new water config.", chat_id.0);

                dialogue.exit().await?;
            }

            Ok(_) => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "⚠️ Ошибка! Процент должен быть целым числом в диапазоне от 1 до 99.\n\n\
                     Пожалуйста, введите корректное число:",
                )
                .await?;
            }

            Err(_) => {
                bot.edit_message_text(
                    chat_id,
                    prev_msg_id,
                    "⚠️ Пожалуйста, введите порог полива только цифрами, без знака % (например: 65).\n\n\
                     Попробуйте еще раз:",
                )
                .await?;
            }
        },

        None => {
            bot.edit_message_text(
                chat_id,
                prev_msg_id,
                "⚠️ Сообщение не содержит текста. Отправьте число от 1 до 99:",
            )
            .await?;
        }
    }

    Ok(())
}

/// Для валидации
pub async fn ensure_water_config(
    bot: &Bot,
    chat_id: ChatId,
    dialogue: &MyDialogue,
    pool: &sqlx::PgPool,
    plant_id: i64,
    prev_msg_id: MessageId,
) -> anyhow::Result<bool> {
    let has_config = crate::db_operations::check_water_config(pool, plant_id)
        .await
        .context("Не удалось проверить наличие water_config в базе данных")?;

    if !has_config {
        bot.edit_message_text(
            chat_id,
            prev_msg_id,
            "⚠️ Настройки полива для этого цветка еще не заданы.\n\
             Чтобы бот мог правильно рассчитывать влажность почвы и присылать напоминания, нам нужно провести быструю калибровку.\n\n\
             ⚖️ Пожалуйста, введите вес ПОЛИТОГО растения в граммах (например: 1250):",
        )
        .await
        .context("Ошибка отправки сообщения о начале калибровки")?;

        dialogue
            .update(MeasurementDialogue::WateringConfig(
                WateringConfigDialog::WaitingWetWeight {
                    prev_msg_id,
                    plant_id,
                },
            ))
            .await
            .context("Не удалось обновить стейт диалога на WaitingWetWeight")?;

        return Ok(false);
    }

    Ok(true)
}
