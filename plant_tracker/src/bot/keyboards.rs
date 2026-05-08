use crate::constants::{REGULAR_PLANT, TROPICAL};

use crate::{constants::SUKKULENT, models::Plant};

use teloxide::types::{
    ButtonRequest, InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup,
};

/// Returns a keyboard with a single "Back to main menu" button.
/// Used as a fallback navigation in most dialogues.
pub fn back_to() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🏠 В главное меню",
        "cancel_action",
    )]])
}

/// Returns the main menu inline keyboard.
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

/// Returns the "My Plants" submenu inline keyboard.
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

/// Returns an inline keyboard with one button per plant.
///
/// Each button's callback data is `"plant_id:plant_name"`.
/// A "Back" button is appended at the bottom with the given `back_to` callback.
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

/// Returns a keyboard for selecting measurement type.
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

/// Returns a keyboard for selecting measurement date.
///
/// Options: today, yesterday, or custom date input.
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

/// Returns a single "Back to My Plants" button.
pub fn back_to_my_plants() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "↩︎  Назад",
        "MyPlants",
    )]])
}

/// Returns a confirmation keyboard for plant deletion.
///
/// Options: confirm deletion or cancel (returns to "My Plants").
pub fn confrim_delete() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Да", "ConfirmDelete"),
        InlineKeyboardButton::callback("❌ Нет", "MyPlants"),
    ]])
}

pub fn plant_type_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🌵 Суккулент", "Succulent")],
        vec![InlineKeyboardButton::callback("🌿 Тропическое", "Tropical")],
        vec![InlineKeyboardButton::callback("🌱 Обычное", "Regular")],
    ])
}

pub fn light_level_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("☀️ Стоит у окна", "window")],
        vec![InlineKeyboardButton::callback(
            "🌥️ В глубине комнаты",
            "shadow",
        )],
    ])
}

pub fn air_circulation_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "🌬️ Есть сквозняк или вентилятор",
            "normal",
        )],
        vec![InlineKeyboardButton::callback(
            "😶 Воздух не движется",
            "stagnant",
        )],
    ])
}

pub fn soil_type_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "🌱 Универсальный грунт",
            "universal",
        )],
        vec![InlineKeyboardButton::callback(
            "🌵 Грунт для кактусов и суккулентов",
            "succulent",
        )],
        vec![InlineKeyboardButton::callback(
            "🌿 Грунт для тропических растений",
            "tropical",
        )],
    ])
}

pub fn geo_button() -> KeyboardMarkup {
    let keyboard = KeyboardButton {
        text: "📍 Отправить локацию".to_string(),
        request: Some(ButtonRequest::Location),
    };
    KeyboardMarkup::default()
        .append_row(vec![keyboard])
        .resize_keyboard()
        .one_time_keyboard()
}
