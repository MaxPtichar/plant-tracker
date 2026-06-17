// mod analytics;
mod bot;

mod db_operations;
mod models;
mod prelude;

// mod analytics_new;
mod analytycs_v2;

use axum::{Router, routing::get};
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    // tracing::info!("Bot has started...");
    // bot::plant_bot().await;

     tokio::spawn(async {
         tracing::info!("Bot has started...");
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
