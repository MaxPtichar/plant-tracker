pub use anyhow::Context;
pub use sqlx::PgPool;

pub use teloxide::dispatching::dialogue::InMemStorage;
pub use teloxide::prelude::*;
pub use teloxide::types::MessageId;

pub use sqlx::postgres::PgPoolOptions;

pub type MyDialogue = Dialogue<MeasurementDialogue, InMemStorage<MeasurementDialogue>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub use crate::bot::MeasurementDialogue;
