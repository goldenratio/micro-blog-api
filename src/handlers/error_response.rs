use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AppErrorResponse {
    pub error_code: i16,
    pub error_message: String,
}
