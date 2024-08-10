use actix_web::{get, HttpResponse, Responder};

#[get("/hello")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}