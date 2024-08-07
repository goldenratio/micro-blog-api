use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::AppState;

use super::error_response::AppErrorResponse;

#[derive(Serialize, Debug, Display)]
pub enum LoginError {
    GenericError,
    InvalidUsernameOrPassword,
}

#[derive(Serialize, Debug, Display)]
pub enum RegisterError {
    GenericError,
    EmailAlreadyExist,
}

#[derive(Deserialize, Debug)]
struct LoginRequestData {
    #[serde(rename = "email")]
    email: String,
    #[serde(rename = "password")]
    password: String,
}

#[derive(Serialize)]
struct LoginSuccessResponse {
    #[serde(rename = "jwtToken")]
    jwt_token: String,
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Deserialize, Debug)]
struct RegisterRequestData {
    #[serde(rename = "email")]
    email: String,
    #[serde(rename = "password")]
    password: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::InvalidUsernameOrPassword => StatusCode::BAD_REQUEST,
            LoginError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            LoginError::InvalidUsernameOrPassword => {
                HttpResponse::build(status).json(AppErrorResponse {
                    errorCode: 0,
                    errorMessage: "Invalid username or password".to_string(),
                })
            }
            LoginError::GenericError => HttpResponse::build(status).json(AppErrorResponse {
                errorCode: 0,
                errorMessage: "Generic Error".to_string(),
            }),
        }
    }
}

impl ResponseError for RegisterError {}

#[post("/login")]
async fn auth_login(param_obj: web::Json<LoginRequestData>) -> Result<impl Responder, LoginError> {
    let payload = param_obj.into_inner();
    println!("/auth {:?}", payload);

    if payload.email.as_str() == "bar@example.com" {
        let response_data = LoginSuccessResponse {
            jwt_token: "123".to_string(),
            user_id: "1456".to_string(),
        };
        return Ok(web::Json(response_data));
    }

    return Err(LoginError::InvalidUsernameOrPassword);
}

#[post("/register")]
async fn auth_register(
    param_obj: web::Json<RegisterRequestData>,
    state: web::Data<AppState>,
) -> Result<impl Responder, RegisterError> {
    let payload = param_obj.into_inner();
    println!("/register {:?}", payload);

    let user_db_service = state.user_db_service.lock().unwrap();
    let uuid = "000-2121";

    match user_db_service.add_user(
        &payload.email,
        &payload.password,
        &payload.display_name,
        uuid,
    ) {
        Ok(_) => {
            return Ok(web::Json({}));
        }
        Err(_) => {
            return Err(RegisterError::EmailAlreadyExist);
        }
    }
}
