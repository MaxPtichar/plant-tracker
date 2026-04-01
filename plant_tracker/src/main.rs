mod analytics;
mod bot;
mod constants;
mod operations;

mod db_operations;
mod models;
// mod models_old;

// mod storage;

#[tokio::main]
async fn main() {
    bot::plant_bot().await;
}


// добавить кнопку вывода pot config для каждого растения