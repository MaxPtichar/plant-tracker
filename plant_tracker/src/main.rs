mod analytics;
mod bot;
mod constants;
mod operations;

mod models_old;
mod storage;
mod models;


#[tokio::main]
async fn main() {
    bot::plant_bot().await;
}
