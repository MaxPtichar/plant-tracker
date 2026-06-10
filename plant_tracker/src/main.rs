mod analytics;
mod bot;
mod constants;
mod db_operations;
mod models;
mod operations;
mod weather;

mod analytics_new;
use axum::{routing::get, Router};
#[tokio::main]
async fn main() {
    tokio::spawn(async {
        bot::plant_bot().await;
    });

    let app = Router::new().route("/", get(|| async { "OK" }));

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .unwrap();

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}