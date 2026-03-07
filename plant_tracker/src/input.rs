use chrono::{Local, NaiveDate};
use std::io;

use crate::constants::COUNT_PLANT_TYPE;

use crate::models::{Measurement, MeasurementType, Plant, PlantType};

use crate::storage::{load, save};

pub fn get_plant() -> (u32, String, PlantType, Vec<Measurement>) {
    // write autoincrement ID

    let id: u32 = get_last_id(&load());
    println!("Hello. Its time to create your first Plant");

    let get_name = read_name();
    let plant_type: PlantType = plants_type();

    let measurements: Vec<Measurement> = vec![measurements()];

    return (id, get_name, plant_type, measurements);
}

fn get_last_id(plant: &[Plant]) -> u32 {
    plant.iter().map(|x| x.id).max().unwrap_or(0) + 1
}

fn read_name() -> String {
    loop {
        println!("Enter Plant's name: ");

        match read_line() {
            Ok(input) if !input.is_empty() => {
                println!("Name: {}", input);
                return input;
            }

            Ok(_) => println!("Name cannot be empty, try again."),
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

fn plants_type() -> PlantType {
    loop {
        println!(
            "Let's choose type of plant(enter the number): \n
    1. Chlorophytum
    2. FicusKinki  
    3. FicusMicrocarpa   
    4. FicusBlackPrince 
    5. Sansevieri 
     "
        );

        match read_line() {
            Ok(input) => match input.parse::<u32>() {
                Ok(1) => return PlantType::Chlorophytum,
                Ok(2) => return PlantType::FicusKinki,
                Ok(3) => return PlantType::FicusMicrocarpa,
                Ok(4) => return PlantType::FicusBlackPrince,
                Ok(5) => return PlantType::Sansevieria,
                Ok(_) => {
                    println!("Enter a number from 1 to {}", COUNT_PLANT_TYPE);
                    continue;
                }
                Err(_) => {
                    println!("Please enter a number");
                    continue;
                }
            },

            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

pub fn read_line() -> Result<String, io::Error> {
    let mut input: String = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn measurements() -> Measurement {
    let weight: f32 = get_weight();

    //реализовать получение даты двумся способами - ввод и текущая дата по нажатию

    let measure_type = get_measurment_type();

    println!("{}", get_current_date());

    Measurement {
        weight,
        date: get_current_date(),
        type_: measure_type,
    }
}

fn get_current_date() -> NaiveDate {
    Local::now().date_naive()
}

fn get_weight() -> f32 {
    loop {
        println!("Enter a weight of plant in gramms");
        match read_line() {
            Ok(input) => match input.parse::<f32>() {
                Ok(value) => {
                    println!("Weight: {}", value);
                    return value;
                }
                Err(_) => {
                    println!("Please enter a number");
                    continue;
                }
            },

            Err(e) => {
                println!("Error: {}", e)
            }
        }
    }
}

fn get_measurment_type() -> MeasurementType {
    loop {
        println!(
            "Choose type of watering: 
        1. Regular 
        2. After watering
        3. After watering with feed"
        );

        match read_line() {
            Ok(input) => match input.parse::<u8>() {
                Ok(1) => return MeasurementType::Regular,
                Ok(2) => return MeasurementType::AfterWatering,
                Ok(3) => return MeasurementType::AfterWateringWithFeed,
                Ok(_) => {
                    println!("Please enter a number from 1 to 3");
                    continue;
                }
                Err(_) => {
                    println!("Please enter a number");
                    continue;
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

// add new measuruments for plant и разбить всю логику на несколько файлов

pub fn add_new_measurement(plants: &mut Vec<Plant>) {
    let choosen_id = input_id(plants);

    let plant = match find_plant(plants, choosen_id) {
        Some(plant) => plant,
        None => {
            println!("id: {} not found", choosen_id);
            return;
        }
    };

    get_new_measurement(plant);
    save(&plants);
}

fn input_id(plant: &mut Vec<Plant>) -> u32 {
    let last_id = get_last_id(plant);
    loop {
        println!("Enter an ID");
        match read_line() {
            Ok(input) => match input.parse::<u32>() {
                Ok(value) if value <= last_id => {
                    println!("id: {}", value);
                    return value;
                }
                Ok(_) => {
                    println!("Enter an id from 1 to {}", last_id);
                    continue;
                }
                Err(_) => {
                    println!("Please enter a number");
                    continue;
                }
            },

            Err(e) => {
                println!("Error: {}", e)
            }
        }
    }
}
pub fn find_plant<'a>(plants: &'a mut Vec<Plant>, choosen_id: u32) -> Option<&'a mut Plant> {
    plants.iter_mut().find(|p| p.id == choosen_id)
}

fn get_new_measurement(plant: &mut Plant) {
    let measure = measurements();
    println!("Added new measures to {}", plant.name);
    plant.measurements.push(measure)
}
