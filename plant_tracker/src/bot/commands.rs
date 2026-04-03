use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::utils::command::{self, BotCommands};

use crate::bot::handlers::plants::{get_all_plants_status, get_list_of_all_plants};
use crate::bot::keyboards::{back_to_my_plants, my_plants_menu};
use crate::bot::keyboards::{main_menu_buttons, plant_keyboard};

use crate::bot::dialogue::PotCreationDialog;
use crate::bot::{HandlerResult, MeasurementDialogue, MyDialogue};
use crate::db_operations;
use crate::operations::format_last_feed;

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
pub enum CallbackCommand {
    MyMeasurements,
    CreatePot,
    CreatePlant,
    PlantList,
    DeletePlant,
}

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
                bot.send_message(msg.chat.id, format!("Пока еще нет ни одного растения🌱"))
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

pub async fn handle_menu_buttons(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    dbg!(&q.data);
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

        "CreatePot" => {
            let plants = db_operations::get_user_plants(&pool, chat_id_i64).await?;
            if plants.is_empty() {
                bot.send_message(chat_id, "Пока еще нет ни одного растения🌱")
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::CreatingPot(
                    PotCreationDialog::ChoosePlantName,
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
            bot.send_message(chat_id, "Введите название растения")
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
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants, "Start"))
                .await?;
        }

        "LastFeed" => {
            let plants = db_operations::recieve_plants_with_last_feed(&pool, chat_id.0).await?;
            let text = format_last_feed(&plants);
            bot.send_message(chat_id, text).await?;
        }

        "Cancel" => {
            dialogue.exit().await?;
            bot.send_message(chat_id, "Отменено").await?;
        }

        _ => {}
    }

    Ok(())
}
