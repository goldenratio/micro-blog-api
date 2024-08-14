use std::sync::Mutex;

use crate::services::user_db_service::UserDbService;

use super::env_settings::EnvSettings;

#[derive(Debug)]
pub struct UserDbState {
    pub service: Mutex<UserDbService>,
}

impl UserDbState {
    pub fn new(env_settings: &EnvSettings) -> Self {
        let user_db_service = UserDbService::connect(&env_settings.db_collection_path).expect("UserDbService error! db_collection_path folder maybe missing");

        Self {
            service: Mutex::from(user_db_service)
        }
    }
}
