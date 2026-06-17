pub use anyhow::Context;
pub use chrono::{DateTime, NaiveDate, TimeZone, Utc};
pub use sqlx::PgPool;
use teloxide::dispatching::dialogue::InMemStorage;
pub use teloxide::prelude::*;

pub use sqlx::postgres::PgPoolOptions;

use crate::bot::MeasurementDialogue;

pub type MyDialogue = Dialogue<MeasurementDialogue, InMemStorage<MeasurementDialogue>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
