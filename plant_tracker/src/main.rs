mod analytics;
mod bot;
mod constants;
mod operations;

mod db_operations;
mod models;

#[tokio::main]
async fn main() {
    bot::plant_bot().await;
}
