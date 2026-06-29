pub use super::dialogue::{MeasurementDialogue, PlantCreationDialogue};

use crate::bot::callbacks::cancel_callback;
use crate::bot::commands::{calendar_handle, handle_menu_buttons};
use crate::bot::dialogue::WateringConfigDialog;
use crate::bot::handlers::{
    delete_last_measure::{receive_answer_measurement, receive_plant_for_delete_measurement},
    delete_plants::{receive_answer, receive_plant_for_delete},
    measurement::receive_plant,
    measurements_record::receive_plant_for_record,
    plant_creation::get_plant_name,
    receive_date, receive_type, receive_weight,
    water_config_creation::{
        receive_dry_soil_weight, receive_plant_for_config, receive_threshold_pct,
        receive_wet_weight,
    },
};
use crate::prelude::*;

pub fn message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    Update::filter_message()
        .branch(
            dptree::case![MeasurementDialogue::WaitingForWeight {
                prev_msg_id,
                plant_id,
            }]
            .endpoint(receive_weight),
        )
        .branch(plant_creation_message_branches())
        .branch(water_config_message_branches())
}

fn plant_creation_message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::CreatingPlant(inner_dialogue)].branch(
        dptree::case![PlantCreationDialogue::WaitingForName { prev_msg_id }]
            .endpoint(get_plant_name),
    )
}

pub fn water_config_message_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::WateringConfig(inner_dialogue)]
        .branch(
            dptree::case![WateringConfigDialog::WaitingWetWeight {
                prev_msg_id,
                plant_id
            }]
            .endpoint(receive_wet_weight),
        )
        .branch(
            dptree::case![WateringConfigDialog::WaitingForDrySoilWeight {
                prev_msg_id,
                plant_id,
                wet_weight
            }]
            .endpoint(receive_dry_soil_weight),
        )
        .branch(
            dptree::case![WateringConfigDialog::WaitingForThresholdPct {
                prev_msg_id,
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
        .branch(dptree::filter(calendar_filter).endpoint(calendar_handle))
        .branch(dptree::filter(is_menu_callback).endpoint(handle_menu_buttons))
        .branch(delete_branches())
        .branch(measurement_branches())
        .branch(delete_branches_measurement())
        .branch(pot_creation_callback_branches())
}

fn pot_creation_callback_branches()
-> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    dptree::case![MeasurementDialogue::WateringConfig(inner_dialogue)].branch(
        dptree::case![WateringConfigDialog::ChoosePlantName { prev_msg_id }]
            .endpoint(receive_plant_for_config),
    )
}
fn delete_branches() -> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription>
{
    dptree::entry()
        .branch(
            dptree::case![MeasurementDialogue::WaitingForPlantDelete { prev_msg_id }]
                .endpoint(receive_plant_for_delete),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForConfirmDelete {
                prev_msg_id,
                plant_id
            }]
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
            dptree::case![MeasurementDialogue::WaitingForMeasurementDelete { prev_msg_id }]
                .endpoint(receive_plant_for_delete_measurement),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForConfirmMeasurementDelete {
                prev_msg_id,
                plant_id
            }]
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
                prev_msg_id,
                plant_id,
            }]
            .endpoint(receive_plant),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForType {
                prev_msg_id,
                plant_id,
                weight
            }]
            .endpoint(receive_type),
        )
        .branch(
            dptree::case![MeasurementDialogue::WaitingForDate {
                prev_msg_id,
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

pub fn calendar_filter(q: CallbackQuery) -> bool {
    q.data
        .as_deref()
        .is_some_and(|d| tg_calendar_widget::handler::is_calendar_callback(d) || d == "calendar")
}
