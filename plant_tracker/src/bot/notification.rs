use crate::analytics::days_until_watering;
use crate::storage::load;
use crate::bot::user::load_chat_id;
use crate::models::watering_status;

use teloxide::Bot;
use teloxide::prelude::Requester;

pub async fn chat_notification (bot: &Bot) {
    let Some(chat_id) = load_chat_id() else {return };
    let plant = load();

    let urgent: Vec<String> = plant.iter().filter_map(|plant| {
        let days = days_until_watering(plant)?;
        if days <= 2.0 {
            Some(format!("🌱 {}: {}", plant.name, watering_status(days)))
            
        } else { None }

}).collect();


    if !urgent.is_empty() {
        let text = format!("💧 Пора поливать:\n{}", urgent.join("\n"));
        let _ = bot.send_message(chat_id, text).await;
    }

    }

    




