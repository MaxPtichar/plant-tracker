use crate::input::get_plant;
use crate::input::{add_new_measurement, read_line, find_plant};
use crate::models::{Plant, watering_status};
use crate::storage::{load, save};
use crate::analytics::{days_until_watering};
use crate::constants::{COUNT_PLANT_TYPE};

fn get_json_load() -> Vec<Plant> {
    load()
}
fn add_new_plant() {
    let mut plants: Vec<Plant> = get_json_load();

    let (id, name, plant_type, measurement) = get_plant();

    println!("New plant {} created!", name);
    let new_plant = Plant::new(id, name, plant_type, measurement);

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

                Ok(3) => get_predicate(plants),

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



fn get_predicate (plants: &Vec<Plant>) {

    for plant in plants {

         let predicate = match days_until_watering(plant) {
        Some(value) => value, 
        None => {println!("{}: do not have data to predicate", plant.name);
        continue;}




     

   
        
    };
    println!("{}: {}",plant.name,  watering_status(predicate));

    }
     

    
    
}
