use crate::bot::keyboards::{confrim_delete, my_plants_menu};

use crate::{db_operations, prelude::*};

pub async fn receive_plant_for_delete(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    prev_msg_id: MessageId,
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
        .update(MeasurementDialogue::WaitingForConfirmDelete {
            prev_msg_id,
            plant_id,
        })
        .await?;

    bot.edit_message_text(
        chat_id,
        prev_msg_id,
        format!("\n\nВы точно хотите удалить ☘️ {plant_name}?\n\n"),
    )
    .reply_markup(confrim_delete())
    .await?;

    Ok(())
}

pub async fn receive_answer(
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
            db_operations::delete_plant(&pool, plant_id).await?;
            bot.edit_message_text(chat_id, prev_msg_id, "Растение удалено")
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
