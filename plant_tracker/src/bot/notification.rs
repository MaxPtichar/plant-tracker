use sqlx::PgPool;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;


use crate::bot::handlers::plants::get_all_plants_status;
use crate::db_operations;

/// Sends watering reminders to all users who have plants that need urgent attention.
///
/// Called daily at 09:00 by [`notification_loop`].
///
/// For each registered user, fetches the watering status of all their plants
/// and sends a notification if any plant is in one of the critical states:
/// - `"Полив просрочен"` — watering is overdue
/// - `"Полить сегодня"` — watering is due today
///
/// Errors per user are logged to stderr and skipped — one failing user
/// does not interrupt notifications for others.


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
