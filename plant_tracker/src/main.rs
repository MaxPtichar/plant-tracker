mod analytics;
mod build_app;
mod constants;
mod input;
mod models;
mod storage;

use crate::storage::{load};

fn main() {
    let mut plants = load();
  
    
    build_app::menu(&mut plants);






    
}
