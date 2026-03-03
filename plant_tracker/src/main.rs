mod analytics;
mod build_app;
mod constants;
mod input;
mod models;
mod storage;

use crate::build_app::get_avr_r_for_each_plant;
use crate::storage::{load};



fn main() {
    let mut plants = load();
    
    get_avr_r_for_each_plant(&mut plants);
    
    build_app::menu(&mut plants);
    





    
}
