use chrono::{DateTime, Days, Local, NaiveDate, Utc};

use teloxide::prelude::*;

use crate::bot::keyboards::main_menu_buttons;
use crate::bot::{HandlerResult, MyDialogue};
use crate::models::MeasurementType;

/// Parses a date string from callback data into [`NaiveDate`].
///
/// # Supported formats
/// - `"today"` — current local date
/// - `"yesterday"` — yesterday's local date
/// - `"YYYY-MM-DD"` — explicit date string
///
/// # Returns
/// - `Some(date)` — successfully parsed
/// - `None` — unrecognized format
pub fn parse_date(q: &str) -> Option<DateTime<Utc>> {
    match q {
        "today" => Some(get_current_date()),
        "yesterday" => Some(get_yesterday_date()),
        _ => None,
    }
}

/// Parses a measurement type string from callback data into [`MeasurementType`].
///
/// # Supported values
/// - `"Regular"` — regular weight measurement
/// - `"AfterWatering"` — measurement taken right after watering
/// - `"AfterWateringWithFeed"` — measurement taken after watering with fertilizer
///
/// # Returns
/// - `Some(MeasurementType)` — successfully parsed
/// - `None` — unrecognized value
pub fn parse_measurement_type(q: &str) -> Option<MeasurementType> {
    match q {
        "Regular" => Some(MeasurementType::Regular),
        "AfterWatering" => Some(MeasurementType::AfterWatering),
        "AfterWateringWithFeed" => Some(MeasurementType::AfterWateringWithFeed),

        _ => None,
    }
}

/// Returns a dptree handler that cancels the current dialogue on `"cancel_action"` callback.
///
/// Exits the dialogue, answers the callback query,
/// and returns the user to the main menu.
pub fn cancel_callback()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some("cancel_action")).endpoint(
        |bot: Bot, dialogue: MyDialogue, q: CallbackQuery| async move {
            dialogue.exit().await?;
            let msg = q.message.as_ref().unwrap();
            let chat_id = msg.chat().id;
            let msg_id = msg.id();

            bot.answer_callback_query(q.id).await?;
            bot.edit_message_text(chat_id, msg_id,"Возвращаюсь в меню.")
                .reply_markup(main_menu_buttons())
                .await?;

            Ok(())
        },
    )
}

fn get_current_date() -> DateTime<Utc> {
    Utc::now()
}
fn get_yesterday_date() -> DateTime<Utc> {
    let today = get_current_date();

    today.checked_sub_days(Days::new(1)).unwrap()
}
