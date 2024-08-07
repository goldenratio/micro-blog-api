mod handlers;
mod services;

use std::sync::Mutex;

use actix_web::{error, get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use handlers::{
    auth::{auth_login, auth_register},
    error_response::AppErrorResponse,
};
use services::user_db_service::UserDbService;

#[derive(Debug)]
pub struct AppState {
    pub user_db_service: Mutex<UserDbService>,
}

impl AppState {
    pub fn new() -> Self {
        let user_db_service = UserDbService::connect().unwrap(); // web::Data::new();
        Self {
            user_db_service: Mutex::from(user_db_service),
        }
    }
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    println!("{}", req_body);
    HttpResponse::Ok().body(req_body)
}

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
                                errorCode: 0,
                                errorMessage: "Invalid request payload".to_string(),
                            }),
                        )
                        .into();
                    }),
            )
            .service(hello)
            .service(echo)
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
