use std::sync::Mutex;

use crate::services::user_db_service::UserDbService;

#[derive(Debug)]
pub struct AppState {
    pub user_db_service: Mutex<UserDbService>,
}

impl AppState {
    pub fn new() -> Self {
        let user_db_service = UserDbService::connect().unwrap(); // web::Data::new();
        Self {
            user_db_service: Mutex::from(user_db_service),
        }
    }
}
