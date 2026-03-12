use chrono::{Days, Local, NaiveDate};

use teloxide::prelude::*;

use crate::bot::keyboards::main_menu_buttons;
use crate::bot::{Command, HandlerResult, MyDialogue};
use crate::models::MeasurementType;

pub fn parse_date(q: &str) -> Option<NaiveDate> {
    match q {
        "today" => Some(get_current_date()),
        "yesterday" => Some(get_yesterday_date()),
        _ => NaiveDate::parse_from_str(q, "%Y-%m-%d").ok(),
    }
}

pub fn parse_main_menu_buttons(q: &str) -> Option<Command> {
    match q {
        "status" => Some(Command::Status),
        "Addmeasurement" => Some(Command::Addmeasurement),
        "LastFeed" => Some(Command::LastFeed),

        _ => None,
    }
}

pub fn parse_measurement_type(q: &str) -> Option<MeasurementType> {
    match q {
        "Regular" => Some(MeasurementType::Regular),
        "AfterWatering" => Some(MeasurementType::AfterWatering),
        "AfterWateringWithFeed" => Some(MeasurementType::AfterWateringWithFeed),

        _ => None,
    }
}

pub fn cancel_callback()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some("cancel_action")).endpoint(
        |bot: Bot, dialogue: MyDialogue, q: CallbackQuery| async move {
            dialogue.exit().await?;
            let chat_id = q.message.as_ref().unwrap().chat().id;
            bot.answer_callback_query(q.id).await?;
            bot.send_message(chat_id, "Действие отменено. Возвращаюсь в меню.")
                .reply_markup(main_menu_buttons())
                .await?;

            Ok(())
        },
    )
}

fn get_current_date() -> NaiveDate {
    Local::now().date_naive()
}
fn get_yesterday_date() -> NaiveDate {
    let today = get_current_date();

    today.checked_sub_days(Days::new(1)).unwrap()
}
