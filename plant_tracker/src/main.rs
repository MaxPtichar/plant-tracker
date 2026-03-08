mod add_new_measurement;
mod analytics;
mod bot;
mod build_app;
mod constants;
mod input;
mod models;
mod storage;



#[tokio::main]
async fn main() {

    bot::plant_bot().await;
}
