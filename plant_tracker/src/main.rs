mod analytics;
mod bot;
mod constants;
mod operations;

mod models;
mod storage;

mod test;

#[tokio::main]
async fn main() {
    bot::plant_bot().await;
}
