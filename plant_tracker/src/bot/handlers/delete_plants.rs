use sqlx::PgPool;
use teloxide::prelude::*;

use crate::bot::keyboards::{confrim_delete, my_plants_menu};

use crate::{
    bot::{HandlerResult, MeasurementDialogue, MyDialogue},
    db_operations,
};

/// Handles plant selection for deletion.
///
/// Parses `plant_id` and `plant_name` from callback data (`"id:name"`),
/// asks the user to confirm deletion via inline keyboard,
/// and transitions the dialogue to [`MeasurementDialogue::WaitingForConfirmDelete`].
pub async fn receive_plant_for_delete(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
) -> HandlerResult {
    if let Some(data) = q.data {
        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();
        dialogue
            .update(MeasurementDialogue::WaitingForConfirmDelete { plant_id })
            .await?;
        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;

        bot.send_message(
            chat_id,
            format!("\n\nВы точно хотите удалить ☘️ {plant_name}?\n\n"),
        )
        .reply_markup(confrim_delete())
        .await?;
    }
    Ok(())
}

/// Handles the user's confirmation or cancellation of plant deletion.
///
/// Called when the dialogue is in [`MeasurementDialogue::WaitingForConfirmDelete`].
///
/// # Transitions
/// - `"ConfirmDelete"` — deletes the plant from the database, exits the dialogue
/// - `"MyPlants"` — cancels deletion, exits the dialogue
///
/// After both actions returns the user to the "My Plants" menu.
pub async fn receive_answer(
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
                db_operations::delete_plant(&pool, plant_id).await?;
                bot.send_message(chat_id, "Растение удалено")
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
