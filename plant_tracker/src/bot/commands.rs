use sqlx::PgPool;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::analytics::days_until_watering;
use crate::bot::callbacks::parse_main_menu_buttons;
use crate::bot::handlers::measurement;
use crate::bot::keyboards::add_new_plant_button;
use crate::bot::keyboards::{main_menu_buttons, plant_keyboard};

use crate::bot::{HandlerResult, MeasurementDialogue, MyDialogue};
use crate::db_operations;
use crate::operations::format_last_feed;
use crate::models::{WateringStatus, watering_status};
use crate::bot::dialogue::PotCreationDialog;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Главное меню")]
    Start,
    #[command(description = "Настроить горшок")]
    CreatePot,
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
        Command::CreatePot => {
            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id_i64).await?;
                 if plants.is_empty() {
        bot.send_message(msg.chat.id, format!("Пока еще нет ни одного растения🌱")).await?;
        dialogue.exit().await?;
        return Ok(());
    }
            dialogue.update(MeasurementDialogue::CreatingPot(PotCreationDialog::ChoosePlantName)).await?;
            bot.send_message(msg.chat.id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
            

        }

        Command::CreatePlant => {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(
                    super::PlantCreationDialogue::WaitingForName,
                ))
                .await?;
            bot.send_message(msg.chat.id, "Введите название растения")
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
        bot.send_message(msg.chat.id, format!("Пока еще нет ни одного растения🌱")).await?;
        dialogue.exit().await?;
        return Ok(());
    }
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(msg.chat.id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
        }

        Command::LastFeed => { let plants = db_operations::recieve_plants_with_last_feed(&pool, msg.chat.id.0).await?;
            bot.send_message(msg.chat.id, format_last_feed(&plants)).await?;
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

        Command::CreatePot => {
            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id_i64).await?;
                 if plants.is_empty() {
        bot.send_message(chat_id, format!("Пока еще нет ни одного растения🌱")).await?;
        dialogue.exit().await?;
        return Ok(());
    }
            dialogue.update(MeasurementDialogue::CreatingPot(PotCreationDialog::ChoosePlantName)).await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
}

        Command::CreatePlant => {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(
                    super::PlantCreationDialogue::WaitingForName,
                ))
                .await?;
            bot.send_message(chat_id, "Введите название растения")
                .await?;
        }

        Command::Status => {
            let text = get_all_plants_status(&pool, chat_id.0).await?;
            bot.send_message(chat_id, text).await?;
        }

        Command::Addmeasurement => {
            let plants: Vec<crate::models::Plant> =
                db_operations::get_user_plants(&pool, chat_id_i64).await?;
             if plants.is_empty() {
        bot.send_message(chat_id, format!("Пока еще нет ни одного растения🌱")).await?;
        dialogue.exit().await?;
        return Ok(());
    }
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;
        }
        Command::LastFeed => {
            
     let plants = db_operations::recieve_plants_with_last_feed(&pool, chat_id.0).await?;
   
    let text = format_last_feed(&plants);
    
    bot.send_message(chat_id, text).await?;
        }

        Command::Cancel => {
            dialogue.exit().await?; // сбрасывает состояние диалога
            bot.send_message(chat_id, "Отменено").await?;
        }
    }
    Ok(())
}


pub async fn get_all_plants_status(pool: &PgPool, chat_id: i64) -> sqlx::Result<String> {

    let plants = db_operations::get_user_plants(&pool, chat_id).await?;
    if plants.is_empty() {
        return Ok(format!("Пока еще нет ни одного растения🌱"));
    }
    let mut result: Vec<String> = Vec::new();

    for plant in &plants {
        let measurements = db_operations::recieve_two_last_measurement(&pool, plant.id).await?;
        let pot = db_operations::get_active_config(&pool, plant.id).await?;
        let target_moisture= plant.target_moisture;
        let after_watering_weight = db_operations::get_last_watering_weight(&pool, plant.id).await?;
        dbg!(&pot);
        dbg!(&after_watering_weight);
        

        let line = match (pot, after_watering_weight) { 
             (Some(pot), Some(after_watering_weight) )=> { 
                match days_until_watering(&measurements, &pot, target_moisture, after_watering_weight) {
                    Some(days) => format!(
                        "🌱 {} — {}",
                        plant.plants_name,
                        watering_status(days)
                    ),
                    None => format!("🌱 {} — нет данных", plant.plants_name),
                }
            }

            
            _ => format!("🌱 {} — не настроено", plant.plants_name),
        };




        result.push(line);

    }

    Ok(result.join("\n"))

}