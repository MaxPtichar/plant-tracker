use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::build_app::get_predicate;

use crate::bot::callbacks::{
     parse_main_menu_buttons};
use crate::bot::keyboards::{
     main_menu_buttons, plant_keyboard,
};
use crate::bot::{HandlerResult, MeasurementDialogue, MyDialogue};
use crate::build_app;
use crate::storage::load;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Главное меню")]
    Start,
    #[command(description = "Когда поливать")]
    Status,
    #[command(description = "Добавить измерение")]
    Addmeasurement,
    #[command(description = "Отменить действие")]
    Cancel,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
) -> HandlerResult {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, "Выбери действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }

        Command::Status => {
            let plants = load();

            bot.send_message(msg.chat.id, get_predicate(&plants))
                .await?;
        }

        Command::Addmeasurement => {
            let plants = load();
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(msg.chat.id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
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
) -> HandlerResult {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data else { return Ok(()) };

    let Some(cmd) = parse_main_menu_buttons(&data) else {
        return Ok(());
    };

    let chat_id = q.message.as_ref().unwrap().chat().id;

    match cmd {
        Command::Start => {
            bot.send_message(chat_id, "Выбери действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }
        Command::Status => {
            let mut plants = load();
            build_app::get_avr_r_for_each_plant(&mut plants);
            bot.send_message(chat_id, get_predicate(&plants)).await?;
        }

        Command::Addmeasurement => {
            let plants = load();
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
        }

        Command::Cancel => {
            dialogue.exit().await?; // сбрасывает состояние диалога
            bot.send_message(chat_id, "Отменено").await?;
        }
    }
    Ok(())
}
