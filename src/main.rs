mod handlers;
mod services;
mod app_data;

use actix_web::{error, middleware, web, App, HttpResponse, HttpServer};
use app_data::app_state::AppState;
use handlers::{
    auth::{auth_login, auth_register},
    error_response::AppErrorResponse, health_check::health_check,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
