use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AppErrorResponse {
    pub errorCode: i16,
    pub errorMessage: String,
}
