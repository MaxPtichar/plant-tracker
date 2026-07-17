pub mod callbacks;
pub mod commands;
pub mod dialogue;
pub mod handlers;
pub mod keyboards;
pub mod notification;
pub mod routing;

use std::time::Duration;

pub use commands::Command;
pub use dialogue::{MeasurementDialogue, PlantCreationDialogue};

use dotenvy::dotenv;
use teloxide::dispatching::Dispatcher;
use teloxide::dispatching::dialogue::{self as tg_dialogue, InMemStorage};
use teloxide::utils::command::BotCommands;

use crate::bot::commands::handle_command;
use crate::bot::notification::chat_notification;
use crate::bot::routing::{callback_branches, message_branches};

use chrono::{Local, Timelike};

use crate::prelude::*;
/// Entry point of the bot. Initialises the database pool, starts the
/// notification loop, registers bot commands and launches the dispatcher.
///
/// # Handler tree
///
/// ## Messages
/// - `/command` → [`handle_command`]
/// - `WaitingForWeight` → [`receive_weight`]
/// - `CreatingPlant / WaitingForName` → [`get_plant_name`]
/// - `CreatingPlant / WaitingForCustomMoisture` → [`get_custom_moisture`]
/// - `CreatingPot / WaitingForPotWeight` → [`recieve_pot_weight`]
/// - `CreatingPot / WaitingForDrySoilWeight` → [`receive_dry_soil_weight`]
///
/// ## Callbacks
/// - `cancel_action` → [`cancel_callback`]
/// - menu buttons → [`handle_menu_buttons`]
/// - `WaitingForPlantDelete` → [`receive_plant_for_delete`]
/// - `WaitingForConfirmDelete` → [`receive_answer`]
/// - `CreatingPot / ChoosePlantName` → [`receive_plant_for_pot`]
/// - `WaitingForPlantRecord` → [`receive_plant_for_record`]
/// - `CreatingPlant / WaitingForMoisture` → [`get_moisture`]
/// - `WaitingForPlant` → [`receive_plant`]
/// - `WaitingForType` → [`receive_type`]
/// - `WaitingForDate` → [`receive_date`]
pub async fn plant_bot() {
    dotenv().ok();

    let bot = Bot::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
        .await
        .expect("Failed to connect to database");

    let bot_clone = bot.clone();
    let pool_clone = pool.clone();

    // tokio::spawn(notification_loop(bot_clone.clone(), pool_clone.clone()));

    bot.set_my_commands(commands::Command::bot_commands())
        .await
        .unwrap();

    let dependencies = dptree::deps![InMemStorage::<MeasurementDialogue>::new(), pool];

    let handler =
        tg_dialogue::enter::<Update, InMemStorage<MeasurementDialogue>, MeasurementDialogue, _>()
            .branch(
                Update::filter_message()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            )
            .branch(message_branches())
            .branch(callback_branches());
    println!("Starting dispatcher");
    Dispatcher::builder(bot, handler)
        .dependencies(dependencies)
        .build()
        .dispatch()
        .await;
}

// / Sends morning watering reminders to all users at 09:00.
// / Checks every 30 seconds, sleeps 60 seconds after sending to avoid double-send.
async fn notification_loop(bot_clone: Bot, pool_clone: PgPool) {
    
    loop {
        tracing::warn!("notification loop started");

        let now = Local::now();
        if now.hour() >= 14 && now.minute() >= 00 {
            chat_notification(&bot_clone, &pool_clone).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        tokio::time::sleep(Duration::from_secs(30)).await;

        tracing::warn!("notification loop ended");
    }
}
