use std::sync::Mutex;

use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use derive_more::Display;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::{
    env_settings::EnvSettings,
    user_db_service::{UserDbError, UserDbService},
};

use super::error_response::AppErrorResponse;

#[derive(Serialize, Debug, Display)]
pub enum AppError {
    InvalidRequestPayload = 10001,
}

#[derive(Serialize, Debug, Display)]
pub enum LoginError {
    InvalidEmailOrPassword = 10011,
}

#[derive(Serialize, Debug, Display)]
pub enum RegisterError {
    GenericError = 10021,
    EmailAlreadyExist,
    DisplayNameAlreadyExist,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LoginRequestData {
    email: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginSuccessResponse {
    jwt_token: String,
    uuid: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RegisterRequestData {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RegisterResponseData {
    uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserClaims {
    pub exp: usize,
    pub uuid: String,
}

impl UserClaims {
    pub fn new(user_jwt_expiration_minutes: i64, uuid: String) -> Self {
        let token_expiry_date =
            (Utc::now() + Duration::minutes(user_jwt_expiration_minutes)).timestamp() as usize;
        Self {
            exp: token_expiry_date,
            uuid: uuid,
        }
    }
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::InvalidEmailOrPassword => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            LoginError::InvalidEmailOrPassword => HttpResponse::build(status)
                .json(AppErrorResponse::from(LoginError::InvalidEmailOrPassword)),
        }
    }
}

impl ResponseError for RegisterError {
    fn status_code(&self) -> StatusCode {
        match self {
            RegisterError::DisplayNameAlreadyExist => StatusCode::BAD_REQUEST,
            RegisterError::EmailAlreadyExist => StatusCode::BAD_REQUEST,
            RegisterError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            RegisterError::DisplayNameAlreadyExist => HttpResponse::build(status).json(
                AppErrorResponse::from(RegisterError::DisplayNameAlreadyExist),
            ),
            RegisterError::EmailAlreadyExist => HttpResponse::build(status)
                .json(AppErrorResponse::from(RegisterError::EmailAlreadyExist)),
            RegisterError::GenericError => HttpResponse::build(status)
                .json(AppErrorResponse::from(RegisterError::GenericError)),
        }
    }
}

impl From<UserDbError> for RegisterError {
    fn from(value: UserDbError) -> Self {
        match value {
            UserDbError::UserWithEmailAlreadyExist => RegisterError::EmailAlreadyExist,
            UserDbError::UserWithDisplayNameAlreadyExist => RegisterError::DisplayNameAlreadyExist,
            _ => RegisterError::GenericError,
        }
    }
}

#[post("/login")]
async fn auth_login(
    param_obj: web::Json<LoginRequestData>,
    user_db_state: web::Data<Mutex<UserDbService>>,
    env_settings: web::Data<EnvSettings>,
) -> Result<impl Responder, LoginError> {
    let payload = param_obj.into_inner();
    log::trace!("/auth {:?}", payload);

    let user_db_service = user_db_state.lock().unwrap();

    if let Ok(password_from_db) = user_db_service.get_password_from_email(&payload.email) {
        if let Ok(valid) = verify(&payload.password, &password_from_db) {
            if !valid {
                return Err(LoginError::InvalidEmailOrPassword);
            }
            // get user struct from DB
            if let Ok(auth_user) = user_db_service.get_user_from_email(&payload.email) {
                let claims = UserClaims::new(
                    env_settings.user_jwt_expiration_minutes,
                    auth_user.uuid.clone(),
                );

                if let Ok(jwt_token) = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(env_settings.user_jwt_secret.as_ref()),
                ) {
                    let response_data = LoginSuccessResponse {
                        jwt_token,
                        uuid: auth_user.uuid,
                    };
                    return Ok(web::Json(response_data));
                } else {
                    log::error!("error generating jwt token for user: {:?}", &payload.email);
                }
            }
        }
    }

    return Err(LoginError::InvalidEmailOrPassword);
}

#[post("/register")]
async fn auth_register(
    param_obj: web::Json<RegisterRequestData>,
    user_db_state: web::Data<Mutex<UserDbService>>,
) -> Result<impl Responder, RegisterError> {
    let payload = param_obj.into_inner();
    log::trace!("/register {:?}", payload);

    let user_db_service = user_db_state.lock().unwrap();
    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();

    let hashed_password = hash(&payload.password, DEFAULT_COST);
    if hashed_password.is_err() {
        return Err(RegisterError::GenericError);
    }

    if let Err(db_err) = user_db_service.add_user(
        &payload.email,
        &hashed_password.unwrap(),
        &payload.display_name,
        &uuid_str,
    ) {
        log::error!("{:?}", db_err);
        return Err(RegisterError::from(db_err));
    }

    return Ok(web::Json(RegisterResponseData { uuid: uuid_str }));
}
