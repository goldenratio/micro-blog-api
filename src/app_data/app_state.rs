use std::sync::Mutex;

use crate::services::user_db_service::UserDbService;

use super::env_settings::EnvSettings;

#[derive(Debug)]
pub struct AppState {
    pub user_db_service: Mutex<UserDbService>,
    pub env_settings: EnvSettings,
}

impl AppState {
    pub fn new() -> Self {
        let env_settings = EnvSettings::new();
        let user_db_service = UserDbService::connect(&env_settings.db_collection_path).unwrap();

        Self {
            user_db_service: Mutex::from(user_db_service),
            env_settings: env_settings,
        }
    }
}
