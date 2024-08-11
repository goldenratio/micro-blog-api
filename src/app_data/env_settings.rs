use std::env;

#[derive(Debug)]
pub struct EnvSettings {
    pub db_collection_path: String,
    pub user_jwt_secret: String,
    pub user_jwt_expiration_minutes: i64,
}

impl EnvSettings {
    pub fn new() -> Self {
        Self {
            db_collection_path: env::var("DB_COLLECTION_FILE_PATH").unwrap(),
            user_jwt_expiration_minutes: env::var("JWT_EXPIRATION_MINUTES")
                .unwrap()
                .parse::<i64>()
                .expect("JWT_EXPIRATION_MINUTES must be a valid i64 number"),
            user_jwt_secret: env::var("USER_JWT_SECRET").unwrap()
        }
    }
}
