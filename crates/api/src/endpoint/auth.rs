use actix_web::{
    get,
    web,
    Responder,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Username {
    pub username: String,
}

#[derive(Deserialize)]
pub struct ServerId {
    pub id: String,
}

#[get("/api/auth/id")]
pub async fn get_server_id(web::Query(Username { username }): web::Query<Username>) -> impl Responder {
    "todo!"
}

#[get("/api/auth/verify")]
pub async fn get_access_token(web::Query(ServerId { id }): web::Query<ServerId>) -> impl Responder {
    "todo!"
}
