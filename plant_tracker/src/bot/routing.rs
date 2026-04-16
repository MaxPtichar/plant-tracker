use std::time::Duration;

pub use super::commands::Command;
pub use super::dialogue::{MeasurementDialogue, PlantCreationDialogue};

use crate::bot::callbacks::cancel_callback;
use crate::bot::commands::{handle_command, handle_menu_buttons};
use crate::bot::dialogue::PotCreationDialog;
use crate::bot::handlers::delete_plants::{receive_answer, receive_plant_for_delete};
use crate::bot::handlers::geo_data::recieve_geo;
use crate::bot::handlers::measurement::{receive_custom_date, receive_plant};
use crate::bot::handlers::measurements_record::receive_plant_for_record;
use crate::bot::handlers::plant_creation::{
    get_air_circ, get_light_level, get_plant_name, get_plant_type,
};
use crate::bot::handlers::pot_creation::{
    receive_dry_soil_weight, receive_plant_for_pot, receive_pot_diameter, receive_soil_type,
    recieve_pot_weight,
};
use crate::bot::handlers::{receive_date, receive_type, receive_weight};
use crate::bot::notification::chat_notification;
use chrono::{Local, Timelike};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub type MyDialogue = Dialogue<MeasurementDialogue, InMemStorage<MeasurementDialogue>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub fn message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    Update::filter_message()
        .branch(
            dptree::case![MeasurementDialogue::WaitingForWeight {
                plant_id,
                plant_name
            }]
            .endpoint(receive_weight),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForCustomDate {
                plant_id,
                weight,
                type_
            }]
            .endpoint(receive_custom_date),
        )
        .branch(plant_creation_message_branches())
        .branch(pot_creation_message_branches())
        .branch(dptree::case![MeasurementDialogue::WaitLocation].endpoint(recieve_geo))
}

fn plant_creation_message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::CreatingPlant(inner_dialogue)]
        .branch(dptree::case![PlantCreationDialogue::WaitingForName].endpoint(get_plant_name))
}

fn pot_creation_message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::CreatingPot(inner_dialogue)]
        .branch(
            dptree::case![PotCreationDialog::WaitingForPotWeight { plant_id }]
                .endpoint(recieve_pot_weight),
        )
        .branch(
            dptree::case![PotCreationDialog::WaitingForDrySoilWeight {
                plant_id,
                pot_weight
            }]
            .endpoint(receive_dry_soil_weight),
        )
        .branch(
            dptree::case![PotCreationDialog::WaitingForDiameter {
                plant_id,
                pot_weight,
                dry_soil_weight
            }]
            .endpoint(receive_pot_diameter),
        )
}

pub fn callback_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    Update::filter_callback_query()
        .branch(cancel_callback())
        .branch(dptree::filter(is_menu_callback).endpoint(handle_menu_buttons))
        .branch(delete_branches())
        .branch(pot_creation_callback_branches())
        .branch(plant_creation_callback_branches())
        .branch(measurement_branches())
}

fn delete_branches() -> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription>
{
    dptree::entry()
        .branch(
            dptree::case![MeasurementDialogue::WaitingForPlantDelete]
                .endpoint(receive_plant_for_delete),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForConfirmDelete { plant_id }]
                .endpoint(receive_answer),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForPlantRecord]
                .endpoint(receive_plant_for_record),
        )
}

fn pot_creation_callback_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::CreatingPot(inner_dialogue)]
        .branch(dptree::case![PotCreationDialog::ChoosePlantName].endpoint(receive_plant_for_pot))
        .branch(
            dptree::case![PotCreationDialog::WaitingForSoilType {
                plant_id,
                pot_weight,
                dry_soil_weight,
                pot_diameter_cm
            }]
            .endpoint(receive_soil_type),
        )
}

fn plant_creation_callback_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::CreatingPlant(inner_dialogue)]
        .branch(
            dptree::case![PlantCreationDialogue::WaitingForPlantType { name }]
                .endpoint(get_plant_type),
        )
        .branch(
            dptree::case![PlantCreationDialogue::WaitingForLightLevel { name, plant_type }]
                .endpoint(get_light_level),
        )
        .branch(
            dptree::case![PlantCreationDialogue::WaitingForAirCirculation {
                name,
                plant_type,
                light_level
            }]
            .endpoint(get_air_circ),
        )
}

fn measurement_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::entry()
        .branch(dptree::case![MeasurementDialogue::WaitingForPlant].endpoint(receive_plant))
        .branch(
            dptree::case![MeasurementDialogue::WaitingForWeight {
                plant_id,
                plant_name
            }]
            .endpoint(receive_plant),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForType { plant_id, weight }]
                .endpoint(receive_type),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForDate {
                plant_id,
                weight,
                type_
            }]
            .endpoint(receive_date),
        )
}

pub fn is_menu_callback(q: CallbackQuery) -> bool {
    q.data.as_deref().map_or(false, |d| {
        d == "status"
            || d == "Addmeasurement"
            || d == "MyPlants"
            || d == "PlantList"
            || d == "DeletePlant"
            || d == "MyMeasurements"
            || d == "LastFeed"
            || d == "Cancel"
            || d == "CreatePlant"
            || d == "CreatePot"
            || d == "Start"
    })
}
