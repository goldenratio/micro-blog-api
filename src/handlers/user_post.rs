use actix_web::{post, web, HttpResponse, Responder, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::app_data::app_state::AppState;

use super::user_auth_token_extractor::UserAuthentication;

#[derive(Serialize, Debug, Display)]
pub enum UserPostError {
    GenericError = 10031,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UserPostRequest {
    title: String,
    post: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UserPostResponse {
    post_id: String,
}

impl ResponseError for UserPostError {}

#[post("/post")]
async fn user_post(
    user_auth: UserAuthentication,
    param_obj: web::Json<UserPostRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, UserPostError> {
    let payload = param_obj.into_inner();
    log::info!("/post {:?}", payload);
    log::info!("user auth: {:?}", user_auth.uuid);
    return Ok(HttpResponse::Ok());
}
