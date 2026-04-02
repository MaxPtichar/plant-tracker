use sqlx::PgPool;
use teloxide::prelude::*;

use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue, PlantCreationDialogue, keyboards::{back_to, get_type_of_moisture, main_menu_buttons}
    },
    db_operations,
};

use crate::bot::dialogue::PotCreationDialog;
pub async fn receive_plant_for_pot(bot: Bot, q: CallbackQuery, dialogue: MyDialogue) -> HandlerResult {
    if let Some(data) = q.data {

        let (plant_id, plant_name) = data
    .split_once(':')
    .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
    .unwrap();


        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;
        bot.send_message(chat_id, format!("☘️ {plant_name}\n\nВведите вес пустого горшка в граммах:"))
            .reply_markup(back_to())
            .await?;

        dialogue
            .update(MeasurementDialogue::CreatingPot(PotCreationDialog::WaitingForPotWeight { plant_id }))
            .await?;

        
    }
    Ok(())
}

pub async fn recieve_pot_weight(bot: Bot, msg: Message, dialogue: MyDialogue, state: PotCreationDialog, ) -> HandlerResult {
    let plant_id = match state {
        PotCreationDialog::WaitingForPotWeight { plant_id } => plant_id, 
        _ => return Ok(()),
    };

     match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(pot_weight) => {

                
                
               bot.send_message(msg.chat.id, format!("\n\nМне нужно знать «нулевую точку» (вес без воды).
Введите общий вес горшка с сухой землей в граммах:»"))
            .await?;

               dialogue
            .update(MeasurementDialogue::CreatingPot(PotCreationDialog::WaitingForDrySoilWeight { plant_id, pot_weight }))
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


pub async fn receive_dry_soil_weight(
    bot: Bot,
    dialogue: MyDialogue,
    pool: PgPool,
    msg: Message,
    state: PotCreationDialog,
) -> HandlerResult {

    let (plant_id, pot_weight) = match state {
        PotCreationDialog::WaitingForDrySoilWeight { plant_id, pot_weight } => (plant_id, pot_weight),
        _ => return Ok(()),
        
    };



    match msg.text() {
        Some(text) => match text.parse::<i64>() {
            Ok(weight_dry) => {
                bot.send_message(
                msg.chat.id,
                format!(
    "✅ Горшок настроен!\n\n\
    🪴 ID растения: {}\n\
    ⚖️ Вес пустого горшка: {} г.\n\
    ⏳ Вес сухой почвы: {} г.\n\
    💧 Общий базовый вес (сухой): {} г.\n\n\
    Теперь при взвешивании я смогу точно рассчитать остаток влаги.", plant_id, pot_weight, weight_dry, pot_weight + weight_dry
            ))
            .await?;
                
                db_operations::create_pot_config(&pool, plant_id, pot_weight, weight_dry).await?;

               dialogue.exit().await?;
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

