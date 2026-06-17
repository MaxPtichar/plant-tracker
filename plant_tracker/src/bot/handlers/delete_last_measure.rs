use crate::bot::keyboards::{confrim_delete, my_plants_menu};
use crate::{bot::MeasurementDialogue, db_operations::get_last_measurement};

use crate::{db_operations, prelude::*};

pub async fn receive_plant_for_delete_measurement(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    if let Some(data) = q.data {
        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();
        dialogue
            .update(MeasurementDialogue::WaitingForConfirmMeasurementDelete { plant_id })
            .await?;
        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;

        if let Some(last_measure) = get_last_measurement(&pool, plant_id).await? {
            let mf = format!(
                "📊 {} г {}",
                last_measure.0,
                last_measure.1.format("%d.%m.%Y")
            );

            bot.send_message(
                chat_id,
                format!("🗑️ Удалить последнее измерение для ☘️ {plant_name}?\n\n{mf}"),
            )
            .reply_markup(confrim_delete())
            .await?;
        } else {
            bot.send_message(
                chat_id,
                format!("📊 Измерений пока нет, выберите другое растение: "),
            )
            .reply_markup(my_plants_menu())
            .await?;
        }
    }
    Ok(())
}

pub async fn receive_answer_measurement(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    plant_id: i64,
    pool: PgPool,
) -> HandlerResult {
    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;
        let chat_id = q.message.unwrap().chat().id;
        match data.as_str() {
            "ConfirmDelete" => {
                db_operations::delete_last_measurement(&pool, plant_id).await?;
                db_operations::set_daily_loss_to_null(&pool, plant_id).await?;

                bot.send_message(chat_id, "Последнее измерение удалено")
                    .reply_markup(my_plants_menu())
                    .await?;
                dialogue.exit().await?;
            }

            "MyPlants" => {
                bot.send_message(chat_id, "Отмена удаления")
                    .reply_markup(my_plants_menu())
                    .await?;
                dialogue.exit().await?;
            }
            _ => {}
        }
    };
    Ok(())
}
