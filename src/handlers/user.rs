use std::{fs::File, path::Path};

use actix_web::{http::StatusCode, post, web, HttpResponse, Responder, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_data::env_settings::EnvSettings;

use super::{error_response::AppErrorResponse, user_auth_token_extractor::UserAuthentication};

#[derive(Serialize, Debug, Display)]
pub enum UserPostError {
    GenericError = 20011,
    PostNotFound,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PostGetByPostIdRequest {
    post_uuid: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PostDataResponse {
    title: String,
    post: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PostListDataResponse {
    posts: Vec<PostDataResponse>,
}

impl ResponseError for UserPostError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserPostError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
            UserPostError::PostNotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        match self {
            UserPostError::GenericError => HttpResponse::build(status)
                .json(AppErrorResponse::from(UserPostError::GenericError)),
            UserPostError::PostNotFound => HttpResponse::build(status)
                .json(AppErrorResponse::from(UserPostError::PostNotFound)),
        }
    }
}

#[post("/post")]
async fn user_post(
    user_auth: UserAuthentication,
    param_obj: web::Json<UserPostRequest>,
    env_settings: web::Data<EnvSettings>,
) -> Result<impl Responder, UserPostError> {
    let payload = param_obj.into_inner();
    log::info!("/post {:?}", payload);

    let user_db_file = format!(
        "{}/user_{}.db",
        env_settings.db_collection_path, user_auth.uuid
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

#[post("/get-post-by-id")]
async fn user_get_post_by_id(
    user_auth: UserAuthentication,
    param_obj: web::Json<PostGetByPostIdRequest>,
    env_settings: web::Data<EnvSettings>,
) -> Result<impl Responder, UserPostError> {
    let payload = param_obj.into_inner();
    log::info!("/get-post-by-id {:?}", payload);

    let user_db_file = format!(
        "{}/user_{}.db",
        env_settings.db_collection_path, user_auth.uuid
    );

    if !Path::new(&user_db_file).exists() {
        log::error!("user DB does not exist! {:?}", user_auth.uuid);
        return Err(UserPostError::GenericError);
    }

    if let Ok(conn) = rusqlite::Connection::open(&user_db_file) {
        if let Ok(mut statement) =
            conn.prepare("SELECT title, post FROM post WHERE uuid=:uuid limit 1;")
        {
            if let Ok(post_iter) = statement.query_map(&[(":uuid", &payload.post_uuid)], |row| {
                let post = PostDataResponse {
                    title: row.get(0)?,
                    post: row.get(1)?,
                };
                Ok(post)
            }) {
                let post_vec: Vec<_> = post_iter.collect();

                if let Some(selected_post) = post_vec.get(0) {
                    if let Ok(post) = selected_post {
                        return Ok(web::Json(post.clone()));
                    }
                }
            }
        }
    }

    return Err(UserPostError::PostNotFound);
}

#[post("/get-posts")]
async fn user_get_posts(
    user_auth: UserAuthentication,
    env_settings: web::Data<EnvSettings>,
) -> Result<impl Responder, UserPostError> {
    log::info!("/get-posts");

    let user_db_file = format!(
        "{}/user_{}.db",
        env_settings.db_collection_path, user_auth.uuid
    );

    if !Path::new(&user_db_file).exists() {
        log::error!("user DB does not exist! {:?}", user_auth.uuid);
        return Err(UserPostError::GenericError);
    }

    if let Ok(conn) = rusqlite::Connection::open(&user_db_file) {
        // todo: pagination
        if let Ok(mut statement) = conn.prepare("SELECT title, post FROM post limit 100;") {
            if let Ok(post_iter) = statement.query_map([], |row| {
                let post = PostDataResponse {
                    title: row.get(0)?,
                    post: row.get(1)?,
                };
                Ok(post)
            }) {
                let post_vec: Vec<_> = post_iter
                    .into_iter()
                    .map(|d| {
                        let post_data = d.unwrap();
                        PostDataResponse {
                            title: post_data.title.clone(),
                            post: post_data.post.clone(),
                        }
                    })
                    .collect();

                return Ok(web::Json(PostListDataResponse { posts: post_vec }));
            }
        }
    }

    return Err(UserPostError::PostNotFound);
}
