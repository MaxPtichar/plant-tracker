pub use super::dialogue::{MeasurementDialogue, PlantCreationDialogue};

use crate::bot::callbacks::cancel_callback;
use crate::bot::commands::handle_menu_buttons;
use crate::bot::dialogue::WateringConfigDialog;
use crate::bot::handlers::delete_last_measure::{
    receive_answer_measurement, receive_plant_for_delete_measurement,
};
use crate::bot::handlers::delete_plants::{receive_answer, receive_plant_for_delete};

use crate::bot::handlers::measurement::{receive_custom_date, receive_plant};
use crate::bot::handlers::measurements_record::receive_plant_for_record;
use crate::bot::handlers::plant_creation::get_plant_name;
use crate::bot::handlers::water_config_creation::{
    receive_dry_soil_weight, receive_plant_for_config, receive_threshold_pct, recieve_wet_weight,
};
use crate::bot::handlers::{receive_date, receive_type, receive_weight};

use teloxide::prelude::*;

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
        .branch(water_config_message_branches())
}

fn plant_creation_message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::CreatingPlant(inner_dialogue)]
        .branch(dptree::case![PlantCreationDialogue::WaitingForName].endpoint(get_plant_name))
}

fn water_config_message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::WateringConfig(inner_dialogue)]
        .branch(
            dptree::case![WateringConfigDialog::WaitingWetWeight { plant_id }]
                .endpoint(recieve_wet_weight),
        )
        .branch(
            dptree::case![WateringConfigDialog::WaitingForDrySoilWeight {
                plant_id,
                wet_weight
            }]
            .endpoint(receive_dry_soil_weight),
        )
        .branch(
            dptree::case![WateringConfigDialog::WaitingForThresholdPct {
                plant_id,
                wet_weight,
                dry_weight
            }]
            .endpoint(receive_threshold_pct),
        )
}

pub fn callback_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    Update::filter_callback_query()
        .branch(cancel_callback())
        .branch(dptree::filter(is_menu_callback).endpoint(handle_menu_buttons))
        .branch(delete_branches())
        .branch(measurement_branches())
        .branch(pot_creation_callback_branches())
        .branch(delete_branches_measurement())
}

fn pot_creation_callback_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::WateringConfig(inner_dialogue)].branch(
        dptree::case![WateringConfigDialog::ChoosePlantName].endpoint(receive_plant_for_config),
    )
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

fn delete_branches_measurement()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::entry()
        .branch(
            dptree::case![MeasurementDialogue::WaitingForMeasurementDelete]
                .endpoint(receive_plant_for_delete_measurement),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForConfirmMeasurementDelete { plant_id }]
                .endpoint(receive_answer_measurement),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForPlantRecord]
                .endpoint(receive_plant_for_record),
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
#[warn(clippy::unnecessary_map_or)]
pub fn is_menu_callback(q: CallbackQuery) -> bool {
    q.data.as_deref().is_some_and(|d| {
        d == "status"
            || d == "addmeasurement"
            || d == "mainmenu"
            || d == "myplants"
            || d == "plantlist"
            || d == "deleteplant"
            || d == "mymeasurements"
            || d == "lastfeed"
            || d == "cancel"
            || d == "createplant"
            || d == "wateringconfig"
            || d == "start"
            || d == "deletelastmeasurement"
            || d == "help"
            || d == "chooseplant"
    })
}