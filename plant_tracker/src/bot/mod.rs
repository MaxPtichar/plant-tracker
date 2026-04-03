pub mod callbacks;
pub mod commands;
pub mod dialogue;
pub mod handlers;
pub mod keyboards;
pub mod notification;

use std::time::Duration;

pub use commands::Command;
pub use dialogue::{MeasurementDialogue, PlantCreationDialogue};

use dotenvy::dotenv;
use sqlx::pool;
use teloxide::dispatching::Dispatcher;
use teloxide::dispatching::dialogue::{self as tg_dialogue, InMemStorage};
use teloxide::utils::command::BotCommands;

use crate::bot::callbacks::cancel_callback;
use crate::bot::commands::{handle_command, handle_menu_buttons};
use crate::bot::dialogue::PotCreationDialog;
use crate::bot::handlers::delete_plants::{receive_answer, receive_plant_for_delete};
use crate::bot::handlers::measurement::receive_plant;
use crate::bot::handlers::measurements_record::receive_plant_for_record;
use crate::bot::handlers::plant_creation::{get_custom_moisture, get_moisture, get_plant_name};
use crate::bot::handlers::pot_creation::{
    receive_dry_soil_weight, receive_plant_for_pot, recieve_pot_weight,
};
use crate::bot::handlers::{receive_date, receive_type, receive_weight};
use crate::bot::notification::chat_notification;
use chrono::{Local, Timelike};
use teloxide::prelude::*;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub type MyDialogue = Dialogue<MeasurementDialogue, InMemStorage<MeasurementDialogue>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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
    tokio::spawn(notification_loop(bot_clone, pool_clone));

    bot.set_my_commands(commands::Command::bot_commands())
        .await
        .unwrap();

    let dependencies = dptree::deps![InMemStorage::<MeasurementDialogue>::new(), pool];

    let handler = tg_dialogue::enter::<
        Update,
        InMemStorage<MeasurementDialogue>,
        MeasurementDialogue,
        _,
    >()
    .branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(handle_command),
    )
    .branch(
        Update::filter_message()
            .branch(
                dptree::case![MeasurementDialogue::WaitingForWeight {
                    plant_id,
                    plant_name
                }]
                .endpoint(receive_weight),
            )
            .branch(
                dptree::case![MeasurementDialogue::CreatingPlant(inner_dialogue)]
                    .branch(
                        dptree::case![PlantCreationDialogue::WaitingForName]
                            .endpoint(get_plant_name),
                    )
                    .branch(
                        dptree::case![PlantCreationDialogue::WaitingForCustomMoisture { name }]
                            .endpoint(get_custom_moisture),
                    ),
            )
            .branch(
                dptree::case![MeasurementDialogue::CreatingPot(inner_dialogue)]
                    .branch(
                        dptree::case![PotCreationDialog::WaitingForPotWeight { plant_id }]
                            .endpoint(recieve_pot_weight),
                    )
                    .branch(
                        dptree::case![PotCreationDialog::WaitingForDrySoilWeight {
                            plant_id,
                            pot_weight
                        }]
                        .endpoint(receive_dry_soil_weight),
                    ),
            ),
    )
    .branch(
        Update::filter_callback_query()
            .branch(cancel_callback())
            .branch(
                dptree::filter(|q: CallbackQuery| {
                    q.data.as_deref().map_or(false, |d| {
                        d == "status"
                            || d == "Addmeasurement"
                            || d == "MyPlants"
                            || d == "PlantList"
                            || d == "DeletePlant"
                            || d == "MyMeasurements"
                            || d == "LastFeed"
                            || d == "Cancel"
                            || d == "CreatePlant"
                            || d == "CreatePot"
                            || d == "Start"
                           
                            
                    })
                })
                .endpoint(handle_menu_buttons),
            )
            .branch(dptree::case![MeasurementDialogue::WaitingForPlantDelete].endpoint(receive_plant_for_delete))
                .branch(dptree::case![MeasurementDialogue::WaitingForConfirmDelete { plant_id }].endpoint(receive_answer))
            
            .branch(
                dptree::case![MeasurementDialogue::CreatingPot(inner_dialogue)].branch(
                    dptree::case![PotCreationDialog::ChoosePlantName]
                        .endpoint(receive_plant_for_pot),
                ),
            )
            .branch(dptree::case![MeasurementDialogue::WaitingForPlantRecord]
    .endpoint(receive_plant_for_record))
            .branch(
                dptree::case![MeasurementDialogue::CreatingPot(inner_dialogue)].branch(
                    dptree::case![PotCreationDialog::ChoosePlantName]
                        .endpoint(receive_plant_for_pot),
                ),
            )
            .branch(
                dptree::case![MeasurementDialogue::CreatingPlant(inner_dialogue)].branch(
                    dptree::case![PlantCreationDialogue::WaitingForMoisture { name }]
                        .endpoint(get_moisture),
                ),
            )

            .branch(dptree::case![MeasurementDialogue::WaitingForPlant].endpoint(receive_plant))
            .branch(
                dptree::case![MeasurementDialogue::WaitingForWeight {
                    plant_id,
                    plant_name
                }]
                .endpoint(receive_plant),
            )
            .branch(
                dptree::case![MeasurementDialogue::WaitingForType { plant_id, weight }]
                    .endpoint(receive_type),
            )
            .branch(
                dptree::case![MeasurementDialogue::WaitingForDate {
                    plant_id,
                    weight,
                    type_
                }]
                .endpoint(receive_date),
            ),
    );
    Dispatcher::builder(bot, handler)
        .dependencies(dependencies)
        .build()
        .dispatch()
        .await;
}

async fn notification_loop(bot_clone: Bot, pool_clone: PgPool) {
    loop {
        let now = Local::now();
        if now.hour() == 21 && now.minute() == 36 {
            chat_notification(&bot_clone, &pool_clone).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
