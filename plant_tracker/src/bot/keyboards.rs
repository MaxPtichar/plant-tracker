use crate::constants::{REGULAR_PLANT, TROPICAL};

use crate::{constants::SUKKULENT, models::Plant};

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
        vec![InlineKeyboardButton::callback(
            "🌿 Мои растения",
            "MyPlants",
        )],
        vec![InlineKeyboardButton::callback(
            "💧 Когда поливать?",
            "status",
        )],
        vec![InlineKeyboardButton::callback(
            "📖 Добавить показания",
            "Addmeasurement",
        )],
        vec![InlineKeyboardButton::callback(
            "🧪 Последняя прикормка",
            "LastFeed",
        )],
    ])
}

pub fn my_plants_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "🌿 Показать мои растения",
            "PlantList",
        )],
        vec![InlineKeyboardButton::callback(
            "📖 Показать мои измерения",
            "MyMeasurements",
        )],
        vec![InlineKeyboardButton::callback(
            "➕ Добавить растение",
            "CreatePlant",
        )],
        vec![InlineKeyboardButton::callback(
            "🪴 Настроить горшок",
            "CreatePot",
        )],
        vec![InlineKeyboardButton::callback(
            "🪏 Удалить растение",
            "DeletePlant",
        )],
        vec![InlineKeyboardButton::callback("↩︎ Назад", "Start")],
    ])
}

pub fn plant_keyboard(plants: &[Plant], back_to: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(
        plants
            .iter()
            .map(|x| {
                vec![InlineKeyboardButton::callback(
                    x.plants_name.clone(),
                    format!("{}:{}", x.id, x.plants_name),
                )]
            })
            .chain(std::iter::once(vec![InlineKeyboardButton::callback(
                "↩︎  Назад",
                back_to,
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

pub fn back_to_my_plants() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "↩︎  Назад",
        "MyPlants",
    )]])
}

pub fn confrim_delete() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Да", "ConfirmDelete"),
        InlineKeyboardButton::callback("❌ Нет", "MyPlants"),
    ]])
}
