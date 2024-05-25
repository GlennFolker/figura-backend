use actix_web::{
    get,
    http::StatusCode,
    web,
    Responder,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    encode_uuid,
    service::auth::AuthService,
};

#[derive(Deserialize)]
pub struct Username {
    pub username: String,
}

#[derive(Deserialize)]
pub struct ServerId {
    pub id: Uuid,
}

#[get("/api/auth/id")]
pub async fn get_server_id(
    web::Query(Username { username }): web::Query<Username>,
    auth: web::Data<AuthService>,
) -> impl Responder {
    encode_uuid(auth.assign_server_id(&username))
}

#[get("/api/auth/verify")]
pub async fn get_access_token(
    web::Query(ServerId { id }): web::Query<ServerId>,
    auth: web::Data<AuthService>,
) -> impl Responder {
    match auth.authenticate(id).await {
        Ok(()) => ("todo!".to_string(), StatusCode::OK),
        Err(e) => (format!("{e}"), StatusCode::INTERNAL_SERVER_ERROR),
    }
}
