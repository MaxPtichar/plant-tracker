use crate::constants::{REGULAR_PLANT, TROPICAL};

use crate::{constants::SUKKULENT, models::Plant as other_plant};

use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn back_to() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🏠 В главное меню",
        "cancel_action",
    )]])
}

pub fn add_new_plant_button() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "Добавить растение",
        "CreatePlant",
    )]])
}

pub fn get_type_of_moisture() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "🌵 Суккулент - 15%",
            SUKKULENT.to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            "🌿 Тропическое - 40%",
            TROPICAL.to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            "🌱 Обычное - 30%",
            REGULAR_PLANT.to_string(),
        )],
        vec![InlineKeyboardButton::callback(
            "✏️ Ввести вручную",
            "custom",
        )],
    ])
}

pub fn main_menu_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        
        vec![InlineKeyboardButton::callback("Добавить растение",
        "CreatePlant",)],
        vec![InlineKeyboardButton::callback("Когда поливать?", "status")],

        vec![InlineKeyboardButton::callback(
            "Добавить показания",
            "Addmeasurement",
        )],
        vec![InlineKeyboardButton::callback(
            "Последняя прикормка",
            "LastFeed",
        )],
    ])
}

pub fn plant_keyboard(plants: &[other_plant]) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        plants
            .iter()
            .map(|x| {
                vec![InlineKeyboardButton::callback(
                    x.plants_name.clone(),
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
