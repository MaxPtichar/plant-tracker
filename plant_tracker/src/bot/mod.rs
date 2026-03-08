pub mod callbacks;
pub mod commands;
pub mod dialogue;
pub mod keyboards;
pub use commands::Command;
pub use dialogue::MeasurementDialogue;

use dotenvy::dotenv;
use teloxide::dispatching::Dispatcher;
use teloxide::dispatching::dialogue::{self as tg_dialogue, InMemStorage};
use teloxide::utils::command::BotCommands;

use teloxide::prelude::*;

use crate::bot::callbacks::cancel_callback;
use crate::bot::commands::{handle_command, handle_menu_buttons};
use crate::bot::dialogue::{receive_plant, receive_type, receive_weight, recieve_date};
use crate::operations;
use crate::storage::{load, save};

pub type MyDialogue = Dialogue<MeasurementDialogue, InMemStorage<MeasurementDialogue>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn plant_bot() {
    dotenv().ok();

    let bot = Bot::from_env();

    bot.set_my_commands(commands::Command::bot_commands())
        .await
        .unwrap();

    let mut plants = load();
    operations::get_avr_r_for_each_plant(&mut plants);
    save(&plants);

    let dependencies = dptree::deps![InMemStorage::<MeasurementDialogue>::new()];

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
        Update::filter_callback_query()
            .branch(cancel_callback())
            .branch(
                dptree::filter(|q: CallbackQuery| {
                    q.data.as_deref().map_or(false, |d| {
                        d == "status" || d == "Addmeasurement" || d == "Cancel"
                    })
                })
                .endpoint(handle_menu_buttons),
            )
            .branch(dptree::case![MeasurementDialogue::WaitingForPlant].endpoint(receive_plant))
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
                .endpoint(recieve_date),
            ),
    )
    .branch(Update::filter_message().branch(
        dptree::case![MeasurementDialogue::WaitingForWeight { plant_id }].endpoint(receive_weight),
    ));

    Dispatcher::builder(bot, handler)
        .dependencies(dependencies)
        .build()
        .dispatch()
        .await;
}
