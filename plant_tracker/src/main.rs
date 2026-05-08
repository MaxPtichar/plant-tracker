mod analytics;
mod bot;
mod constants;
mod db_operations;
mod models;
mod operations;
mod weather;

mod analytics_new;

#[tokio::main]
async fn main() {
    bot::plant_bot().await;
}
