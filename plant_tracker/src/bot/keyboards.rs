use crate::models::Plant;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn back_to() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🏠 В главное меню",
        "cancel_action",
    )]])
}

pub fn main_menu_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("Когда поливать?", "status")],
        vec![InlineKeyboardButton::callback(
            "Добавить показания",
            "Addmeasurement",
        )],
        vec![InlineKeyboardButton::callback(
            "Последняя прикормка",
            "LastFeed",
        )],
        vec![InlineKeyboardButton::callback("Отмена", "Cancel")],
    ])
}

pub fn plant_keyboard(plants: &[Plant]) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        plants
            .iter()
            .map(|x| {
                vec![InlineKeyboardButton::callback(
                    x.name.clone(),
                    x.id.to_string(),
                )]
            })
            .chain(std::iter::once(vec![InlineKeyboardButton::callback(
                "❌ Отмена",
                "cancel_action",
            )]))
            .collect::<Vec<_>>(),
    )
}

pub fn measurement_type_keyboard() -> InlineKeyboardMarkup {
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

pub fn date_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("Сегодня", "today")],
        vec![InlineKeyboardButton::callback("Вчера", "yesterday")],
        vec![InlineKeyboardButton::callback(
            "Ввести свою дату",
            "your_date",
        )],
    ])
}
