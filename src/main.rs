mod app_data;
mod handlers;
mod services;

use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use app_data::app_state::AppState;
use dotenv::dotenv;
use handlers::{
    auth::{auth_login, auth_register},
    error_response::AppErrorResponse,
    health_check::health_check,
    user_post::user_post,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let app_state = web::Data::new(AppState::new());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(1024)
                    .error_handler(|err, _req| {
                        return error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json(AppErrorResponse {
                                error_code: 0,
                                error_message: "Invalid request payload".to_string(),
                            }),
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
            .service(web::scope("/user").service(user_post))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
