use actix_web::{get, http::StatusCode, web, Responder, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    encode_uuid,
    endpoint::header::{
        AccessToken,
        UserAgent,
    },
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

#[get("/api")]
pub async fn refresh_access_token(
    req: HttpRequest,
    _: web::Header<UserAgent>,
    web::Header(token): web::Header<AccessToken>,
    auth: web::Data<AuthService>,
) -> impl Responder {
    if auth.refresh_access_token(&req, token.0).await {
        ("hello from `figura-backend`!", StatusCode::OK)
    } else {
        ("invalid or expired access token", StatusCode::UNAUTHORIZED)
    }
}

#[get("/api/auth/id")]
pub async fn assign_server_id(
    web::Query(Username { username }): web::Query<Username>,
    auth: web::Data<AuthService>,
) -> impl Responder {
    encode_uuid(auth.assign_server_id(&username))
}

#[get("/api/auth/verify")]
pub async fn obtain_access_token(
    req: HttpRequest,
    web::Query(ServerId { id }): web::Query<ServerId>,
    auth: web::Data<AuthService>,
) -> impl Responder {
    match auth.obtain_access_token(&req, id).await {
        Some(token) => (encode_uuid(token), StatusCode::OK),
        None => ("invalid server ID".to_string(), StatusCode::UNAUTHORIZED),
    }
}
