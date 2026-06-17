use std::vec;

use crate::models::Plant;

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
        vec![
        InlineKeyboardButton::callback(
            "🌿 Мои растения",
            "myplants",
        ),
        InlineKeyboardButton::callback(
            "💧 Когда поливать?",
            "status",
        ),
    ],
    
       vec![InlineKeyboardButton::callback(
            "📖 Добавить показания",
            "addmeasurement",
        ),
        InlineKeyboardButton::callback(
            "🧪 Последняя прикормка",
            "lastfeed",
        ),
    ]])
}

/// Returns the "My Plants" submenu inline keyboard.
pub fn my_plants_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
    
        vec![
            InlineKeyboardButton::callback("🌿 Мои растения", "plantlist"),
            InlineKeyboardButton::callback("📖 Мои измерения", "mymeasurements"),
        ],
      
        vec![
            InlineKeyboardButton::callback("➕ Добавить растение", "createplant"),
            InlineKeyboardButton::callback("⚙️ Настроить полив", "wateringconfig"),
        ],
     
        vec![
            InlineKeyboardButton::callback("🪏 Удалить растение", "deleteplant"),
            InlineKeyboardButton::callback("🗑 Удалить замер", "deletelastmeasurement"),
        ],
    
        vec![
            InlineKeyboardButton::callback("↩︎ Назад в главное меню", "mainmenu")
        ],
    ])
}

/// Returns an inline keyboard with one button per plant.
///
/// Each button's callback data is `"plant_id:plant_name"`.
/// A "Back" button is appended at the bottom with the given `back_to` callback.
pub fn plant_keyboard(plants: &[Plant], back_to: &str) -> InlineKeyboardMarkup {
    let mut row:Vec<Vec<InlineKeyboardButton>> =
        plants
            .iter()
            .map(|x| {
                InlineKeyboardButton::callback(
                    x.plants_name.clone(),
                    format!("{}:{}", x.id, x.plants_name),
                )
            })


          
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| chunk.to_vec())
            .collect();



             
    ;


    row.push(vec![InlineKeyboardButton::callback("↩︎ Назад", back_to)]);

    InlineKeyboardMarkup::new(row)
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
        "myplants",
    )]])
}

/// Returns a confirmation keyboard for plant deletion.
///
/// Options: confirm deletion or cancel (returns to "My Plants").
pub fn confrim_delete() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Да", "confirmdelete"),
        InlineKeyboardButton::callback("❌ Нет", "myplants"),
    ]])
}


pub fn first_page() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
        InlineKeyboardButton::callback("📖 Как это работает?", "help"),
        InlineKeyboardButton::callback("🪴 Добавить растение", "createplant"),]
    ])
}




