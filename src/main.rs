mod app_data;
mod handlers;
mod services;

use std::sync::Mutex;

use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use app_data::env_settings::EnvSettings;
use dotenv::dotenv;
use handlers::{
    auth::{auth_login, auth_register, AppError},
    error_response::AppErrorResponse,
    health_check::health_check,
    user::{user_get_post_by_id, user_get_posts, user_post},
};
use services::user_db_service::UserDbService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let env_settings = EnvSettings::new();
    let user_db_service = UserDbService::connect(&env_settings.db_collection_path)
        .expect("UserDbService error! db_collection_path folder maybe missing");
    let user_db_state = web::Data::new(Mutex::new(user_db_service));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(env_settings.clone()))
            .app_data(user_db_state.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(1024)
                    .error_handler(|err, _req| {
                        return error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest()
                                .json(AppErrorResponse::from(AppError::InvalidRequestPayload)),
                        )
                        .into();
                    }),
            )
            .service(health_check)
            .service(
                web::scope("/auth")
                    .service(auth_login)
                    .service(auth_register),
            )
            .service(
                web::scope("/user")
                    .service(user_post)
                    .service(user_get_posts)
                    .service(user_get_post_by_id),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
