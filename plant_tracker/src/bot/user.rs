use teloxide::types::{ ChatId};
use std::fs;

pub fn save_chat_id(chat_id: ChatId) {
    
    fs::create_dir_all("data").unwrap();
    
    fs::write("data/chat_id.txt", chat_id.0.to_string()).unwrap();

}


pub fn load_chat_id() -> Option<ChatId> {
    match fs::read_to_string("data/chat_id.txt") {
        Ok(content) => content.trim().parse::<i64>().ok().map(ChatId),
        Err(_) => None,
        
    }
    
    
}