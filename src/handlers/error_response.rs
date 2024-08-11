use serde::Serialize;

use super::{
    auth::{AppError, LoginError, RegisterError},
    user_post::UserPostError,
};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorResponse {
    pub error_code: u16,
    pub error_message: String,
}

impl From<AppError> for AppErrorResponse {
    fn from(value: AppError) -> AppErrorResponse {
        match value {
            AppError::InvalidRequestPayload => {
                return AppErrorResponse {
                    error_code: AppError::InvalidRequestPayload as u16,
                    error_message: "Invalid request payload".to_string(),
                };
            }
        }
    }
}

impl From<LoginError> for AppErrorResponse {
    fn from(value: LoginError) -> AppErrorResponse {
        match value {
            LoginError::GenericError => {
                return AppErrorResponse {
                    error_code: LoginError::GenericError as u16,
                    error_message: "Unknown generic error".to_string(),
                };
            }
            LoginError::InvalidEmailOrPassword => {
                return AppErrorResponse {
                    error_code: LoginError::InvalidEmailOrPassword as u16,
                    error_message: "Invalid email or password".to_string(),
                };
            }
        }
    }
}

impl From<RegisterError> for AppErrorResponse {
    fn from(value: RegisterError) -> AppErrorResponse {
        match value {
            RegisterError::GenericError => {
                return AppErrorResponse {
                    error_code: RegisterError::GenericError as u16,
                    error_message: "Unknown generic error".to_string(),
                };
            }
            RegisterError::DisplayNameAlreadyExist => {
                return AppErrorResponse {
                    error_code: RegisterError::DisplayNameAlreadyExist as u16,
                    error_message: "Display name already exist".to_string(),
                };
            }
            RegisterError::EmailAlreadyExist => {
                return AppErrorResponse {
                    error_code: RegisterError::DisplayNameAlreadyExist as u16,
                    error_message: "An account with email already exist".to_string(),
                };
            }
        }
    }
}

impl From<UserPostError> for AppErrorResponse {
    fn from(value: UserPostError) -> AppErrorResponse {
        match value {
            UserPostError::GenericError => {
                return AppErrorResponse {
                    error_code: UserPostError::GenericError as u16,
                    error_message: "Unknown generic error".to_string(),
                };
            }
        }
    }
}
