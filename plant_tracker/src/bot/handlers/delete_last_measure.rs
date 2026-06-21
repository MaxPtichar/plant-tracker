use teloxide::types::MessageId;

use crate::bot::keyboards::{confrim_delete, my_plants_menu};
use crate::db_operations;
use crate::prelude::*;

pub async fn receive_plant_for_delete_measurement(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    prev_msg_id: MessageId,
    pool: PgPool,
) -> HandlerResult {
    let Some(data) = q.data else { return Ok(()) };
    let Some(message) = q.message.as_ref() else {
        return Ok(());
    };
    let chat_id = message.chat().id;

    let Some((plant_id, plant_name)) = data
        .split_once(':')
        .and_then(|(id, name)| id.parse::<i64>().ok().map(|id| (id, name.to_string())))
    else {
        return Ok(());
    };

    bot.answer_callback_query(q.id).await?;

    dialogue
        .update(MeasurementDialogue::WaitingForConfirmMeasurementDelete {
            prev_msg_id,
            plant_id,
        })
        .await?;

    if let Some(last_measure) = db_operations::get_last_measurement(&pool, plant_id).await? {
        let mf = format!(
            "📊 {} г {}",
            last_measure.0,
            last_measure.1.format("%d.%m.%Y")
        );

        bot.edit_message_text(
            chat_id,
            prev_msg_id,
            format!("🗑️ Удалить последнее измерение для ☘️ {plant_name}?\n\n{mf}"),
        )
        .reply_markup(confrim_delete())
        .await?;
    } else {
        bot.edit_message_text(
            chat_id,
            prev_msg_id,
            format!("📊 {plant_name}: змерений пока нет."),
        )
        .reply_markup(my_plants_menu())
        .await?;
    }

    Ok(())
}

pub async fn receive_answer_measurement(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (prev_msg_id, plant_id): (MessageId, i64),
    pool: PgPool,
) -> HandlerResult {
    let Some(data) = q.data else { return Ok(()) };
    bot.answer_callback_query(q.id).await?;
    let Some(message) = q.message.as_ref() else {
        return Ok(());
    };
    let chat_id = message.chat().id;

    match data.as_str() {
        "confirmdelete" => {
            db_operations::delete_last_measurement(&pool, plant_id).await?;
            db_operations::set_daily_loss_to_null(&pool, plant_id).await?;

            bot.edit_message_text(chat_id, prev_msg_id, "Последнее измерение удалено")
                .reply_markup(my_plants_menu())
                .await?;
            dialogue.exit().await?;
        }
        "myplants" => {
            bot.edit_message_text(chat_id, prev_msg_id, "Отмена удаления")
                .reply_markup(my_plants_menu())
                .await?;
            dialogue.exit().await?;
        }
        _ => {}
    }

    Ok(())
}
