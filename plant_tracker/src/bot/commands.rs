use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::analytycs_v2::format_last_feed;
use crate::bot::dialogue::WateringConfigDialog;
use crate::bot::handlers::plants::{get_all_plants_status, get_list_of_all_plants};
use crate::bot::keyboards::{back_to, back_to_my_plants, my_plants_menu};
use crate::bot::keyboards::{main_menu_buttons, plant_keyboard};

use crate::bot::{HandlerResult, MeasurementDialogue, MyDialogue};
use crate::db_operations::{self};


/// Bot commands available via `/` in Telegram.
///
/// Callback-only actions (`CreatePot`, `CreatePlant`, etc.) are handled
/// separately in [`handle_menu_buttons`] and are not exposed as commands.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Главное меню")]
    Start,
    #[command(description = "Мои растения")]
    MyPlants,
    #[command(description = "Когда поливать")]
    Status,
    #[command(description = "Добавить измерение")]
    Addmeasurement,
    #[command(description = "Последняя прикормка")]
    LastFeed,
    #[command(description = "Отменить действие")]
    Cancel,
}

/// Handles bot commands sent via `/command` syntax.
///
/// # Commands
/// - `/start` — registers the user and shows the main menu
/// - `/myplants` — shows the "My Plants" submenu
/// - `/status` — shows watering status for all plants
/// - `/addmeasurement` — starts the measurement recording dialogue
/// - `/lastfeed` — shows the last fertilizer application date per plant
/// - `/cancel` — exits the current dialogue
pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    let username = msg.chat.username();
    let chat_id_i64 = msg.chat.id.0;

    match cmd {
        Command::Start => {
            db_operations::create_user(&pool, chat_id_i64, username).await?;
            bot.send_message(msg.chat.id, "Выберите действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }

        Command::MyPlants => {
            bot.send_message(msg.chat.id, "Выберите действие: ")
                .reply_markup(my_plants_menu())
                .await?;
        }

        Command::Status => {
            let text = get_all_plants_status(&pool, msg.chat.id.0).await?;
            bot.send_message(msg.chat.id, text).await?;
        }

        Command::Addmeasurement => {
            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id_i64).await?;
            if plants.is_empty() {
                bot.send_message(msg.chat.id, "Пока еще нет ни одного растения🌱".to_string())
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(msg.chat.id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants, "Start"))
                .await?;
        }

        Command::LastFeed => {
            let plants = db_operations::recieve_plants_with_last_feed(&pool, msg.chat.id.0).await?;
            bot.send_message(msg.chat.id, format_last_feed(&plants))
                .await?;
        }

        Command::Cancel => {
            dialogue.exit().await?; // сбрасывает состояние диалога
            bot.send_message(msg.chat.id, "Отменено").await?;
        }
    }
    Ok(())
}

/// Handles inline keyboard button presses from the main and submenu screens.
///
/// Matches `q.data` string directly against known callback values:
/// - `"Start"` — main menu
/// - `"MyPlants"` — my plants submenu
/// - `"PlantList"` — list of all plants with details
/// - `"MyMeasurements"` — measurement history, starts plant selection dialogue
/// - `"DeletePlant"` — delete plant, starts plant selection dialogue
/// - `"CreatePot"` — pot configuration, starts pot creation dialogue
/// - `"CreatePlant"` — starts plant creation dialogue
/// - `"status"` — watering status for all plants
/// - `"Addmeasurement"` — starts measurement recording dialogue
/// - `"LastFeed"` — last fertilizer application date
/// - `"Cancel"` — exits the current dialogue
pub async fn handle_menu_buttons(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data else { return Ok(()) };

    let chat_id = q.message.as_ref().unwrap().chat().id;
    let username = q.from.username.as_deref();
    let chat_id_i64 = chat_id.0;

    match data.as_str() {
        "Start" => {
            db_operations::create_user(&pool, chat_id_i64, username).await?;

            bot.send_message(chat_id, "Выбери действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }

        "MyPlants" => {
            bot.send_message(chat_id, "Выберите действие: ")
                .reply_markup(my_plants_menu())
                .await?;
        }

        "PlantList" => {
            let text = get_list_of_all_plants(&pool, chat_id.0).await?;
            bot.send_message(chat_id, text)
                .reply_markup(back_to_my_plants())
                .await?;
        }

        "MyMeasurements" => {
            let plants = db_operations::get_user_plants(&pool, chat_id_i64).await?;
            if plants.is_empty() {
                bot.send_message(chat_id, "Пока нет растений 🌱").await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForPlantRecord)
                .await?;
            bot.send_message(chat_id, "Выбери растение:")
                .reply_markup(plant_keyboard(&plants, "MyPlants"))
                .await?;
        }

        "DeletePlant" => {
            let plants = db_operations::get_user_plants(&pool, chat_id_i64).await?;
            if plants.is_empty() {
                bot.send_message(chat_id, "Пока нет растений 🌱").await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForPlantDelete)
                .await?;
            bot.send_message(chat_id, "\nВыбери растение, которое хотите удалить:\n")
                .reply_markup(plant_keyboard(&plants, "MyPlants"))
                .await?;
        }

        "WateringConfig" => {
            let plants = db_operations::get_user_plants(&pool, chat_id_i64).await?;
            if plants.is_empty() {
                bot.send_message(chat_id, "Пока еще нет ни одного растения🌱")
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WateringConfig(
                    WateringConfigDialog::ChoosePlantName,
                ))
                .await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants, "MyPlants"))
                .await?;
        }

        "CreatePlant" => {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(
                    super::PlantCreationDialogue::WaitingForName,
                ))
                .await?;
            bot.send_message(chat_id, "🪴 Добавление нового растения\nВведите его название:")
                .await?;
        }

        "status" => {

            let text = get_all_plants_status(&pool, chat_id.0).await?;
            bot.send_message(chat_id, text).await?;
        }

        "Addmeasurement" => {



            let plants = db_operations::get_user_plants(&pool, chat_id_i64).await?;
            
            if plants.is_empty() {
                bot.send_message(chat_id, "Пока еще нет ни одного растения🌱")
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }

          

            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;

            bot.send_message(chat_id, "Выберите растение: ")
                .reply_markup(plant_keyboard(&plants, "Start"))
                .await?;
        }

        "LastFeed" => {
            let plants = db_operations::recieve_plants_with_last_feed(&pool, chat_id.0).await?;
            let text = format_last_feed(&plants);
            bot.send_message(chat_id, text)
                .reply_markup(back_to())
                .await?;
        }

        "Cancel" => {
            dialogue.exit().await?;
            bot.send_message(chat_id, "Отменено").await?;
        }

        _ => {}
    }

    Ok(())
}
