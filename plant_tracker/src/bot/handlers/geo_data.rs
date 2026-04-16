use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;

use crate::bot::keyboards::{confrim_delete, main_menu_buttons, my_plants_menu};

use crate::{
    bot::{HandlerResult, MeasurementDialogue, MyDialogue},
    db_operations,
};

/// Handles plant selection for deletion.
///
/// Parses `plant_id` and `plant_name` from callback data (`"id:name"`),
/// asks the user to confirm deletion via inline keyboard,
/// and transitions the dialogue to [`MeasurementDialogue::WaitingForConfirmDelete`].
pub async fn recieve_geo(
    bot: Bot,
    msg: Message,
    pool: PgPool,
    dialogue: MyDialogue,
) -> HandlerResult {
    if let Some(location) = msg.location() {
        let lat = location.latitude;
        let lon = location.longitude;

        db_operations::create_geo(&pool, lat, lon, msg.chat.id.0).await?;

        bot.send_message(
            msg.chat.id, 
            format!("Спасибо! Координаты получены: {}, {}. Теперь я знаю погоду у вас за окном.", lat, lon) )
          .reply_markup(ReplyMarkup::kb_remove())
        .await?;

        bot.send_message(
            msg.chat.id, 
            format!("Выберете действие") )
          .reply_markup(main_menu_buttons())
        .await?;

        dialogue.exit().await?;
        
    }
    else {
        bot.send_message(msg.chat.id, "Пожалуйста, используйте кнопку для отправки локации.").await?;
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
