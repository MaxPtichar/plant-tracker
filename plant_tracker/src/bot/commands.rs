use sqlx::PgPool;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::bot::callbacks::parse_main_menu_buttons;
use crate::bot::keyboards::add_new_plant_button;
use crate::bot::keyboards::{main_menu_buttons, plant_keyboard};
use crate::bot::user::save_chat_id;
use crate::bot::{HandlerResult, MeasurementDialogue, MyDialogue};
use crate::db_operations;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Главное меню")]
    Start,
    #[command(description = "Добавить новое растение")]
    CreatePlant,
    #[command(description = "Когда поливать")]
    Status,
    #[command(description = "Добавить измерение")]
    Addmeasurement,
    #[command(description = "Последняя прикормка")]
    LastFeed,
    #[command(description = "Отменить действие")]
    Cancel,
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
            bot.send_message(msg.chat.id, "Выбери действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }

        Command::CreatePlant => {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(super::PlantCreationDialogue::WaitingForName))
                .await?;
            bot.send_message(msg.chat.id, "Введите название растения")
                .await?;
        }

        Command::Status => {
            // let plants = load();

            // bot.send_message(msg.chat.id, get_predicate(&plants))
            //     .await?;
        }

        Command::Addmeasurement => {
            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id_i64).await?;
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(msg.chat.id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
        }

        Command::LastFeed => {
            // let plants = load();
            // bot.send_message(msg.chat.id, last_feed(&plants)).await?;
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
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data else { return Ok(()) };

    let Some(cmd) = parse_main_menu_buttons(&data) else {
        return Ok(());
    };

    let chat_id = q.message.as_ref().unwrap().chat().id;
    let username = q.from.username.as_deref();
    let chat_id_i64 = chat_id.0;

    match cmd {
        Command::Start => {
            db_operations::create_user(&pool, chat_id_i64, username).await?;
            bot.send_message(chat_id, "Выбери действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }

        Command::CreatePlant => {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(super::PlantCreationDialogue::WaitingForName))
                .await?;
            bot.send_message(chat_id, "Введите название растения")
                .await?;
        }

        Command::Status => {
            // let mut plants = load();
            // get_avr_r_for_each_plant(&mut plants);
            // bot.send_message(chat_id, get_predicate(&plants)).await?;
        }

        Command::Addmeasurement => {
            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id_i64).await?;
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
        }
        Command::LastFeed => {
            // let plants = load();
            // bot.send_message(chat_id, last_feed(&plants)).await?;
        }

        Command::Cancel => {
            dialogue.exit().await?; // сбрасывает состояние диалога
            bot.send_message(chat_id, "Отменено").await?;
        }
    }
    Ok(())
}
