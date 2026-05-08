use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;

use crate::bot::keyboards::main_menu_buttons;

use crate::{
    bot::{HandlerResult, MyDialogue},
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
            format!(
                "Спасибо! Координаты получены: {}, {}. Теперь я знаю погоду у вас за окном.",
                lat, lon
            ),
        )
        .reply_markup(ReplyMarkup::kb_remove())
        .await?;

        bot.send_message(msg.chat.id, format!("Выберете действие"))
            .reply_markup(main_menu_buttons())
            .await?;

        dialogue.exit().await?;
    } else {
        bot.send_message(
            msg.chat.id,
            "Пожалуйста, используйте кнопку для отправки локации.",
        )
        .await?;
    }

    Ok(())
}
