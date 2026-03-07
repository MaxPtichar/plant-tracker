mod analytics;
mod bot_api;
mod build_app;
mod constants;
mod input;
mod models;
mod storage;
mod  add_new_measurement;

use crate::bot_api::plant_bot;
use crate::build_app::get_avr_r_for_each_plant;
use crate::storage::load;

#[tokio::main]
async fn main() {
    // let mut plants = load();

    // get_avr_r_for_each_plant(&mut plants);

    // let menu = build_app::menu(&mut plants);

    plant_bot().await;
}
