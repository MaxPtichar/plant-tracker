use teloxide::utils::command::BotCommands;

use crate::analytycs_v2::format_last_feed;
use crate::bot::dialogue::WateringConfigDialog;
use crate::bot::handlers::plants::{get_all_plants_status, get_list_of_all_plants};
use crate::bot::keyboards::{back_to, back_to_my_plants, first_page, my_plants_menu};
use crate::bot::keyboards::{main_menu_buttons, plant_keyboard};

use crate::db_operations;
use crate::prelude::*;

/// Bot commands available via `/` in Telegram.
///
/// Callback-only actions (`CreatePot`, `CreatePlant`, etc.) are handled
/// separately in [`handle_menu_buttons`] and are not exposed as commands.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Главное меню")]
    MainMenu,
    #[command(description = "Добавить измерение")]
    Addmeasurement,
    #[command(description = "Когда поливать")]
    Status,
    #[command(description = "Последняя прикормка")]
    LastFeed,
    #[command(description = "Помощь")]
    Help,
    #[command(description = "Старт")]
    Start,
    #[command(description = "Отменить действие")]
    Cancel,
}

/// Handles bot commands sent via `/command` syntax.
///
/// # Commands
/// - `/start` — registers the user and shows the main menu
/// - `/myplants` — shows the "My Plants" submenu
/// - `/status` — shows watering status for all plants
/// - `/addmeasurement` — starts the measurement recording dialogue
/// - `/lastfeed` — shows the last fertilizer application date per plant
/// - `/cancel` — exits the current dialogue
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
                .reply_markup(first_page())
                .await?;
        }

        Command::MainMenu => {
            bot.send_message(msg.chat.id, "Выберите действие: ")
                .reply_markup(main_menu_buttons())
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
                bot.send_message(msg.chat.id, "Пока еще нет ни одного растения🌱".to_string())
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;
            bot.send_message(msg.chat.id, "Выбери растение: ")
                .reply_markup(plant_keyboard(&plants, "Start"))
                .await?;
        }

        Command::LastFeed => {
            let plants = db_operations::recieve_plants_with_last_feed(&pool, msg.chat.id.0).await?;
            bot.send_message(msg.chat.id, format_last_feed(&plants))
                .await?;
        }

        Command::Help => {
            let fmt = help_text();

            let chat_id = msg.chat.id;
            bot.send_message(chat_id, fmt).await?;
        }

        Command::Cancel => {
            dialogue.exit().await?; // сбрасывает состояние диалога
            bot.send_message(msg.chat.id, "Отменено").await?;
        }
    }
    Ok(())
}

/// Handles inline keyboard button presses from the main and submenu screens.
///
/// Matches `q.data` string directly against known callback values:
/// - `"Start"` — main menu
/// - `"myplants"` — my plants submenu
/// - `"PlantList"` — list of all plants with details
/// - `"MyMeasurements"` — measurement history, starts plant selection dialogue
/// - `"DeletePlant"` — delete plant, starts plant selection dialogue
/// - `"CreatePot"` — pot configuration, starts pot creation dialogue
/// - `"CreatePlant"` — starts plant creation dialogue
/// - `"status"` — watering status for all plants
/// - `"Addmeasurement"` — starts measurement recording dialogue
/// - `"LastFeed"` — last fertilizer application date
/// - `"Cancel"` — exits the current dialogue
pub async fn handle_menu_buttons(
    bot: Bot,
    q: CallbackQuery,
    dialogue: MyDialogue,
    pool: PgPool,
) -> HandlerResult {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(data) = q.data else { return Ok(()) };

    let chat_id = q.message.as_ref().unwrap().chat().id;
    let username = q.from.username.as_deref();
    let chat_id_i64 = chat_id.0;

    let message = q.message.as_ref().unwrap();
    let msg_id = message.id();
    let plants = db_operations::get_user_plants(&pool, chat_id_i64).await?;

    match data.as_str() {
        "start" => {
            db_operations::create_user(&pool, chat_id_i64, username).await?;

            bot.edit_message_text(chat_id, msg_id, "Выберите действие: ")
                .reply_markup(first_page())
                .await?;
        }

        "mainmenu" => {
            bot.edit_message_text(chat_id, msg_id, "Выберите действие: ")
                .reply_markup(main_menu_buttons())
                .await?;
        }
        "myplants" => {
            bot.edit_message_text(chat_id, msg_id, "Выберите действие: ")
                .reply_markup(my_plants_menu())
                .await?;
        }

        "plantlist" => {
            let text = get_list_of_all_plants(&pool, chat_id.0).await?;
            bot.edit_message_text(chat_id, msg_id, text)
                .reply_markup(back_to_my_plants())
                .await?;
        }

        "mymeasurements" => {
            if plants.is_empty() {
                bot.edit_message_text(chat_id, msg_id, "Пока нет растений 🌱")
                    .await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForPlantRecord)
                .await?;
            bot.edit_message_text(chat_id, msg_id, "Выбери растение:")
                .reply_markup(plant_keyboard(&plants, "myplants"))
                .await?;
        }

        "deleteplant" => {
            if plants.is_empty() {
                bot.edit_message_text(chat_id, msg_id, "Пока нет растений 🌱")
                    .await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForPlantDelete {
                    prev_msg_id: msg_id,
                })
                .await?;
            bot.edit_message_text(
                chat_id,
                msg_id,
                "\nВыберите растение, которое хотите удалить:\n",
            )
            .reply_markup(plant_keyboard(&plants, "myplants"))
            .await?;
        }

        "wateringconfig" => {
            if plants.is_empty() {
                bot.edit_message_text(chat_id, msg_id, "Пока еще нет ни одного растения🌱")
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WateringConfig(
                    WateringConfigDialog::ChoosePlantName {
                        prev_msg_id: msg_id,
                    },
                ))
                .await?;
            bot.edit_message_text(chat_id, msg_id, "Выберите растение: ")
                .reply_markup(plant_keyboard(&plants, "myplants"))
                .await?;
        }

        "createplant" => {
            dialogue
                .update(MeasurementDialogue::CreatingPlant(
                    super::PlantCreationDialogue::WaitingForName {
                        prev_msg_id: msg_id,
                    },
                ))
                .await?;
            bot.edit_message_text(
                chat_id,
                msg_id,
                "🪴 Добавление нового растения\nВведите его название:",
            )
            .await?;
        }

        "status" => {
            let text = get_all_plants_status(&pool, chat_id.0).await?;

            bot.edit_message_text(chat_id, msg_id, text)
                .reply_markup(back_to())
                .await?;
        }

        "addmeasurement" => {
            if plants.is_empty() {
                bot.edit_message_text(chat_id, msg_id, "Пока еще нет ни одного растения🌱")
                    .await?;
                dialogue.exit().await?;
                return Ok(());
            }

            dialogue
                .update(MeasurementDialogue::WaitingForPlant)
                .await?;

            bot.edit_message_text(chat_id, msg_id, "Выберите растение: ")
                .reply_markup(plant_keyboard(&plants, "mainmenu"))
                .await?;
        }

        "lastfeed" => {
            let plants_last_feed =
                db_operations::recieve_plants_with_last_feed(&pool, chat_id.0).await?;
            let text = format_last_feed(&plants_last_feed);
            bot.edit_message_text(chat_id, msg_id, text)
                .reply_markup(back_to())
                .await?;
        }

        "cancel" => {
            dialogue.exit().await?;
            bot.edit_message_text(chat_id, msg_id, "Отменено").await?;
        }

        "deletelastmeasurement" => {
            if plants.is_empty() {
                bot.edit_message_text(chat_id, msg_id, "Пока нет растений 🌱")
                    .await?;
                return Ok(());
            }
            dialogue
                .update(MeasurementDialogue::WaitingForMeasurementDelete {
                    prev_msg_id: msg_id,
                })
                .await?;
            bot.edit_message_text(
                chat_id,
                msg_id,
                "\nВыбери растение для того, чтобы удалить последнее измерение.",
            )
            .reply_markup(plant_keyboard(&plants, "myplants"))
            .await?;
        }

        "help" => {
            let fmt = help_text();

            let message = q.message.as_ref().unwrap();
            let msg_id = message.id();
            bot.edit_message_text(chat_id, msg_id, fmt).await?;
        }

        "chooseplant" => {
            let message = q.message.as_ref().unwrap();
            let msg_id = message.id();
            bot.edit_message_text(chat_id, msg_id, "Выберите растение: ")
                .reply_markup(plant_keyboard(&plants, "myplants"))
                .await?;
        }

        _ => {}
    }

    Ok(())
}

fn help_text() -> String {
    "🌿 Как устроен полив по весу?

Этот бот предсказывает идеальное время полива и присылает уведомления. Чтобы всё заработало, нужно пройти 4 простых шага:

1️⃣ Создайте растение
Нажмите команду /createplant или выберите соответствующий пункт в меню.

2️⃣ Настройте параметры полива
(Включается автоматически после создания растения, либо через: /mainmenu ➡️ Мои Растения ➡️ Настроить полив).

Для расчётов боту нужны 3 показателя:
💧 Максимальный вес — вес горшка сразу после обильного полива.
🍂 Сухой вес — вес горшка, когда в грунте совсем не осталось влаги.
💡 Как прикинуть сухой вес? Вспомните вес пустого горшка аналогичного размера и прибавьте к нему вес сухой земли (обычно 1 л сухого торфяного грунта весит около 300–400 грамм). Или просто взвесьте растение, когда оно явно попросит пить.
📊 Процент влаги для полива — порог (например, 20-30%), ниже которого растение начинает страдать. Информацию для конкретного цветка можно подсмотреть в интернете.

3️⃣ Добавьте первый замер
Взвесьте горшок и перейдите: /mainmenu ➡️ Добавить показания ➡️ Введите вес в граммах. Бот предложит 3 варианта:
• ⚖️ Обычное взвешивание — текущий контроль (полива не было).
• 🚿 После полива — если только что напоили цветок чистой водой.
• 🧪 После полива с прикормкой — если добавили удобрения.

4️⃣ Следите за статусом!
Посмотреть состояние всех цветов можно через /status или пункт меню «Когда поливать?».

🤖 Важно: Бот начнёт строить графики и делать прогнозы сразу после того, как вы зафиксируете первый полив (вариант «После полива»), а затем внесете хотя бы один промежуточный («Обычный») замер.

Жмите /createplant, чтобы добавить свой первый цветок! ✨ ".to_string()
}
