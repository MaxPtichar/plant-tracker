use crate::analytics::{days_until_watering, get_avg_r};
use crate::constants::COUNT_PLANT_TYPE;
use crate::input::get_plant;
use crate::input::{add_new_measurement, find_plant, read_line};
use crate::models::{Plant, watering_status};
use crate::storage::{load, save};

fn get_json_load() -> Vec<Plant> {
    load()
}
fn add_new_plant() {
    let mut plants: Vec<Plant> = get_json_load();

    let (id, name, plant_type, measurement) = get_plant();

    println!("New plant {} created!", name);
    let new_plant = Plant::new(id, name, plant_type, measurement, None);

    plants.push(new_plant);

    save(&plants);
}

pub fn menu(plants: &mut Vec<Plant>) {
    loop {
        println!(
            "\n------Menu-------
    1. Create plants
    2. Add wieght to plant
    3. Get days untill watering
    4. exit "
        );

        match read_line() {
            Ok(input) => match input.parse::<u8>() {
                Ok(1) => add_new_plant(),

                Ok(2) => add_new_measurement(plants),

                Ok(3) => println!("{:?}", get_predicate(plants)),

                Ok(4) => break,
                Ok(_) => println!("Enter a number from 1 to 4"),

                Err(_) => {
                    println!("Error, consider enter a number");
                    continue;
                }
            },

            Err(e) => println!("Error: {}", e),
        }
    }
}

pub fn get_avr_r_for_each_plant(plants: &mut [Plant]) {
    for plant in plants.iter_mut() {
        let avg = get_avg_r(plant);
        plant.update_avg_r(avg);
    }
}

pub fn get_predicate(plants: &Vec<Plant>) -> String {
    let mut lines: Vec<String> = Vec::new();

    for plant in plants {
        let predicate = match days_until_watering(plant) {
            Some(value) => format!("🌱 {}: {}", plant.name, watering_status(value)),
            None => {
                format!("🌱 {}: нет данных", plant.name)
            }
        };
        lines.push(predicate);
    }

    lines.join("\n")
}
