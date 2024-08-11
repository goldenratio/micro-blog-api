use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use chrono::{Duration, Utc};
use derive_more::Display;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_data::app_state::AppState, services::user_db_service::UserDbError};

use super::error_response::AppErrorResponse;

// const JWT_EXPIRATION_MINUTES: i64 = 18;
// const JWT_SECRET: &str = "AhYd]wP7sLn6c0bD9^X_onyWkVgY^b";

#[derive(Serialize, Debug, Display)]
pub enum LoginError {
    GenericError = 10011,
    InvalidEmailOrPassword,
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
    user_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RegisterRequestData {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Serialize)]
struct UserClaims {
    exp: usize,
    uuid: String,
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
            LoginError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            LoginError::InvalidEmailOrPassword => HttpResponse::build(status)
                .json(AppErrorResponse::from(LoginError::InvalidEmailOrPassword)),
            LoginError::GenericError => {
                HttpResponse::build(status).json(AppErrorResponse::from(LoginError::GenericError))
            }
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
    state: web::Data<AppState>,
) -> Result<impl Responder, LoginError> {
    let payload = param_obj.into_inner();
    log::trace!("/auth {:?}", payload);

    let user_db_service = state.user_db_service.lock().unwrap();

    if let Ok(auth_user) =
        user_db_service.get_user_from_email_and_password(&payload.email, &payload.password)
    {
        let claims = UserClaims::new(state.env_settings.user_jwt_expiration_minutes, auth_user.uuid.clone());

        if let Ok(jwt_token) = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.env_settings.user_jwt_secret.as_ref()),
        ) {
            let response_data = LoginSuccessResponse {
                jwt_token: jwt_token,
                user_id: auth_user.uuid,
            };
            return Ok(web::Json(response_data));
        } else {
            log::error!("error generating jwt token for user: {:?}", &payload.email);
        }
        // return Err(LoginError::GenericError);
    }

    return Err(LoginError::InvalidEmailOrPassword);
}

#[post("/register")]
async fn auth_register(
    param_obj: web::Json<RegisterRequestData>,
    state: web::Data<AppState>,
) -> Result<impl Responder, RegisterError> {
    let payload = param_obj.into_inner();
    log::trace!("/register {:?}", payload);

    let user_db_service = state.user_db_service.lock().unwrap();
    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();

    if let Err(db_err) = user_db_service.add_user(
        &payload.email,
        &payload.password,
        &payload.display_name,
        &uuid_str,
    ) {
        log::error!("{:?}", db_err);
        return Err(RegisterError::from(db_err));
    }

    return Ok(HttpResponse::Ok());
}
