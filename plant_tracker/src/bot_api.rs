use dotenvy::dotenv;
use teloxide::{dptree::HandlerResult, macros::BotCommands, prelude::*};

use crate::build_app::get_predicate;
use crate::storage::load;
use crate::{analytics::*, build_app};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Status,
}

pub async fn plant_bot() {
    dotenv().ok();

    let bot = Bot::from_env();

    Command::repl(bot, handle_command).await;
}

async fn handle_command(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Status => {
            let mut plants = load();
            build_app::get_avr_r_for_each_plant(&mut plants);
            bot.send_message(msg.chat.id, get_predicate(&plants))
                .await?;
        }
    }
    Ok(())
}
