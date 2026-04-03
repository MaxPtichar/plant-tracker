use sqlx::{PgPool, pool};
use teloxide::prelude::*;

use crate::bot::keyboards::{back_to, back_to_my_plants, confrim_delete, my_plants_menu};
use crate::models::MeasurementType;
use crate::{
    bot::{
        HandlerResult, MeasurementDialogue, MyDialogue,
        callbacks::{parse_date, parse_measurement_type},
        keyboards::{date_keyboard, measurement_type_keyboard, plant_keyboard},
    },
    db_operations,
};


pub async fn receive_plant_for_delete(bot: Bot, q: CallbackQuery, dialogue: MyDialogue) -> HandlerResult {
    if let Some(data) = q.data {

        let (plant_id, plant_name) = data
            .split_once(':')
            .map(|(id, name)| (id.parse::<i64>().unwrap(), name.to_string()))
            .unwrap();
         dialogue
            .update(MeasurementDialogue::WaitingForConfirmDelete {
                plant_id,
            })
            .await?;
        bot.answer_callback_query(q.id).await?;

        let chat_id = q.message.unwrap().chat().id;

       
        bot.send_message(
            chat_id,
            format!("\n\nВы точно хотите удалить ☘️ {plant_name}?\n\n"),
        )
        .reply_markup(confrim_delete())
        .await?;

    }
    Ok(())
}

pub async fn receive_answer(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
    plant_id: i64,
    pool: PgPool,
) -> HandlerResult {

    if let Some(data) = q.data {
        bot.answer_callback_query(q.id).await?;
        let chat_id = q.message.unwrap().chat().id;
        match data.as_str() {
            "ConfirmDelete" => { db_operations::delete_plant(&pool, plant_id).await?;
                bot.send_message(chat_id, "Растение удалено").reply_markup(my_plants_menu())
            .await?; dialogue.exit().await?;}

            "MyPlants" => { bot.send_message(chat_id, "Отмена удаления").reply_markup(my_plants_menu())
            .await?; dialogue.exit().await?;}
            _ => {}

        }
        

      
};
Ok(())
}