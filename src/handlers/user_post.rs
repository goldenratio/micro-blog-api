use std::{fs::File, path::Path};

use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_data::app_state::AppState;

use super::{error_response::AppErrorResponse, user_auth_token_extractor::UserAuthentication};

#[derive(Serialize, Debug, Display)]
pub enum UserPostError {
    GenericError = 20011,
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
    post_uuid: String,
}

impl ResponseError for UserPostError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserPostError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            UserPostError::GenericError => HttpResponse::build(status)
                .json(AppErrorResponse::from(UserPostError::GenericError)),
        }
    }
}

#[post("/post")]
async fn user_post(
    user_auth: UserAuthentication,
    param_obj: web::Json<UserPostRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, UserPostError> {
    let payload = param_obj.into_inner();
    log::info!("/post {:?}", payload);

    let user_db_file = format!(
        "{}/user_{}.db",
        state.env_settings.db_collection_path, user_auth.uuid
    );

    if !Path::new(&user_db_file).exists() {
        // Only create the file if it doesn't exist
        if let Err(err) = File::create(&user_db_file) {
            log::error!("{:?}", err);
            return Err(UserPostError::GenericError);
        }
    }

    let res = match rusqlite::Connection::open(&user_db_file) {
        Ok(conn) => {
            match conn.execute(
                "CREATE TABLE IF NOT EXISTS post (
                 id      INTEGER PRIMARY KEY,
                 title   TEXT NOT NULL,
                 post    TEXT NOT NULL,
                 uuid    TEXT NOT NULL UNIQUE
          )",
                (), // empty list of parameters.
            ) {
                Ok(_) => {
                    let post_uuid = Uuid::new_v4();
                    let post_uuid_str = post_uuid.to_string();
                    if let Err(err) = conn.execute(
                        "INSERT INTO post (title, post, uuid) VALUES (?1, ?2, ?3)",
                        (&payload.title, &payload.post, &post_uuid_str),
                    ) {
                        log::error!("insert post error, {:?}", err);
                        Err(UserPostError::GenericError)
                    } else {
                        Ok(post_uuid_str.clone())
                    }
                }
                Err(_) => Err(UserPostError::GenericError),
            }
        }
        Err(_) => Err(UserPostError::GenericError),
    };

    match res {
        Ok(post_uuid) => Ok(web::Json(UserPostResponse { post_uuid })),
        Err(err) => Err(err),
    }
}
