use std::default;

use chrono::{Days, Local, NaiveDate};
use dotenvy::dotenv;
use teloxide::dispatching::Dispatcher;
use teloxide::dispatching::dialogue::{self, GetChatId, InMemStorage};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::utils::command::BotCommands;
use teloxide::{prelude::*};


use crate::add_new_measurement::add_new_measurement;
use crate::build_app::get_predicate;
use crate::models::{Measurement, MeasurementType, Plant};
use crate::storage::{load, save};
use crate::{analytics::*, build_app};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Главное меню")]
    Start,
    #[command(description = "Когда поливать")]

    Status,
    #[command(description = "Добавить измерение")]

    Addmeasurement,
    #[command(description = "Отменить действие")]

    Cancel,
}

#[derive(Clone, Default)]
enum MeasurementDialogue {
    #[default]
    WaitingForPlant,
    WaitingForWeight {
        plant_id: u32,
    },
    WaitingForType {
        plant_id: u32,
        weight: f32,
    },
    WaitingForDate {
        plant_id: u32,
        weight: f32,
        type_: MeasurementType,
    },
}

type MyDialogue = Dialogue<MeasurementDialogue, InMemStorage<MeasurementDialogue>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn plant_bot() {
    dotenv().ok();

    let bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands()).await.unwrap();

    let mut plants = load();
            build_app::get_avr_r_for_each_plant(&mut plants);
            save(&plants);

    let dependencies = dptree::deps![InMemStorage::<MeasurementDialogue>::new()];

    let handler = dialogue::enter::<Update, InMemStorage<MeasurementDialogue>, MeasurementDialogue, _>()
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
                    q.data.as_deref().map_or(false, |d| d == "status" || d == "Addmeasurement" || d == "Cancel")
                })
                .endpoint(handle_menu_buttons)
            )
            .branch(dptree::case![MeasurementDialogue::WaitingForPlant].endpoint(receive_plant))
            .branch(
                dptree::case![MeasurementDialogue::WaitingForType { plant_id, weight }]
                    .endpoint(receive_type),
            )
            .branch(dptree::case![MeasurementDialogue::WaitingForDate { plant_id, weight, type_ }]
                .endpoint(recieve_date)

            )
            
            
            ,
    )
    .branch(
        Update::filter_message().branch(
            dptree::case![MeasurementDialogue::WaitingForWeight { plant_id }].endpoint(receive_weight),
        ),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dependencies)
        .build()
        .dispatch()
        .await;
}

async fn handle_command(
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
             dialogue.update(MeasurementDialogue::WaitingForPlant).await?;
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


async fn handle_menu_buttons(bot: Bot, q: CallbackQuery, dialogue: MyDialogue) -> HandlerResult {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data else { return Ok(())};

    let Some(cmd) = parse_main_menu_buttons(&data) else { return Ok(())};

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
            bot.send_message(chat_id, get_predicate(&plants))
                .await?;
        }

        Command::Addmeasurement => {
            let plants = load();
            dialogue.update(MeasurementDialogue::WaitingForPlant).await?;
            bot.send_message(chat_id,  "Выбери растение: ")
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

//get weight
async fn receive_weight(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    plant_id: u32,
) -> HandlerResult {
    match msg.text() {
        Some(text) => match text.parse::<f32>() {
            Ok(weight) => {
                dialogue
                    .update(MeasurementDialogue::WaitingForType { plant_id, weight })
                    .await?;

                bot.send_message(msg.chat.id, "Выбери тип измерений: ")
                    .reply_markup(measurement_type_keyboard())
                    .await?;
            }

            Err(_) => {
                bot.send_message(msg.chat.id, "Введите число").await?;
            }
        },

        None => {
            bot.send_message(msg.chat.id, "Введите вес в граммах")
                .await?;
        }
    }

    Ok(())
}

// buttons
fn back_to() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "🏠 В главное меню",
            "cancel_action",
        )]])
}

fn main_menu_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("Когда поливать?", "status")],
        vec![InlineKeyboardButton::callback(
            "Добавить показания",
            "Addmeasurement",
        )],
        vec![InlineKeyboardButton::callback("Отмена", "Cancel")],
    ])
}

fn plant_keyboard(plants: &[Plant]) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        plants
            .iter()
            .map(|x| {
                vec![InlineKeyboardButton::callback(
                    x.name.clone(),
                    x.id.to_string(),
                )]
            }).chain(std::iter::once(vec![InlineKeyboardButton::callback("❌ Отмена", "cancel_action")]))
            
            .collect::<Vec<_>>(),
            
    )
}

fn measurement_type_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "Обычное взвешивание",
            "Regular",
        )],
        vec![InlineKeyboardButton::callback(
            "После полива",
            "AfterWatering",
        )],
        vec![InlineKeyboardButton::callback(
            "После полива с прикормкой",
            "AfterWateringWithFeed",
        )],
    ])
}

fn date_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "Сегодня",
            "today",
        )],
        vec![InlineKeyboardButton::callback(
            "Вчера",
            "yesterday",
        )],
        vec![InlineKeyboardButton::callback(
            "Ввести свою дату",
            "your_date",
        )],
    ])
}

// callbacks

fn cancel_callback() -> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some("cancel_action"))
                .endpoint(|bot: Bot, dialogue: MyDialogue, q: CallbackQuery| async move {
                    dialogue.exit().await?;
                    let chat_id = q.message.as_ref().unwrap().chat().id;
                    bot.answer_callback_query(q.id).await?;
                    bot.send_message(chat_id, "Действие отменено. Возвращаюсь в меню.")
                    .reply_markup(main_menu_buttons())
                    .await?;
                    
                    Ok(())
                })
    
    
}




async fn receive_type(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    (plant_id, weight): (u32, f32),
) -> HandlerResult {
    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;

        if let Some(type_) = parse_measurement_type(&data) {
            dialogue
                .update(MeasurementDialogue::WaitingForDate {
                    plant_id,
                    weight,
                    type_,
                })
                .await?;

            let chat_id = q.message.unwrap().chat().id;
            bot.send_message(chat_id, "Выберите дату: ")
                .reply_markup(date_keyboard())
                .await?;
        }
    }
    Ok(())
}

async fn recieve_date(
    bot: Bot, 
    dialogue: MyDialogue,
    q: CallbackQuery,
    (plant_id, weight, type_): (u32, f32, MeasurementType)) -> HandlerResult  {
        let chat_id = q.message.unwrap().chat().id;

        if let Some(data) = q.data { 
            bot.answer_callback_query(q.id).await?;
        
        if let Some(date) = parse_date(&data) {
            bot.send_message(chat_id, format!("Записано: {} г., тип полива: {:?}, дата: {}", weight, type_, date)).await?;

            let mut plants = load();
            add_new_measurement(&mut plants, plant_id, weight, date, type_);


            dialogue.update(MeasurementDialogue::WaitingForPlant).await?;
            bot.send_message(chat_id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants))
                .await?;

        } 
        else {   bot.send_message(chat_id, "Неверный формат даты. Попробуйте еще раз").await?;}

    }

    Ok(())
    }



fn get_current_date() -> NaiveDate {
    Local::now().date_naive()
}
fn get_yesterday_date() -> NaiveDate{
    let today = get_current_date();

    today.checked_sub_days(Days::new(1)).unwrap()
    
}

fn parse_date(q: &str) -> Option<NaiveDate> {
    match q {
        "today" => Some(get_current_date()),
        "yesterday" => Some(get_yesterday_date()), 
        _ => NaiveDate::parse_from_str(q, "%Y-%m-%d").ok(),
        
    }
}







fn parse_main_menu_buttons(q: &str) -> Option<Command> {
    match q {
        "status" => Some(Command::Status),
        "Addmeasurement" => Some(Command::Addmeasurement),

        _ => None,
    }
}

fn parse_measurement_type(q: &str) -> Option<MeasurementType> {
    match q {
        "Regular" => Some(MeasurementType::Regular),
        "AfterWatering" => Some(MeasurementType::AfterWatering),
        "AfterWateringWithFeed" => Some(MeasurementType::AfterWateringWithFeed),

        _ => None,
    }
}

// get id plant
async fn receive_plant(bot: Bot, q: CallbackQuery, dialogue: MyDialogue) -> HandlerResult {
    if let Some(data) = q.data {
        let plant_id: u32 = data.parse().unwrap();
        bot.answer_callback_query(q.id).await?;

        dialogue
            .update(MeasurementDialogue::WaitingForWeight { plant_id })
            .await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(chat_id, "Введите вес в граммах")
        .reply_markup(back_to())
        
        .await?;
    }
    Ok(())
}
