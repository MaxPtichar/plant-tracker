use sqlx::PgPool;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;

use crate::bot::handlers::plants::get_all_plants_status;
use crate::db_operations;

pub async fn chat_notification(bot: &Bot, pool: &PgPool) {
    let users = match db_operations::get_all_users(pool).await {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Failed to get users: {e}");
            return;
        }
    };

    for user in users {
        let chat_id = ChatId(user.id);

        let status = match get_all_plants_status(pool, user.id).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to get status for {}: {e}", user.id);
                continue;
            }
        };
        if status.contains("Полив просрочен") || status.contains("Полить сегодня")
        {
            let _ = bot
                .send_message(chat_id, format!("💧 Напоминание о поливе:\n{}", status))
                .await;
        }
    }
}
