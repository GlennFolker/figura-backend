use crate::Ws;
use std::{
    convert::Infallible,
    hash::Hasher,
    str::FromStr,
};
use actix_web::{
    error::ParseError,
    http::header,
    web,
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use fxhash::FxHasher;
use rand::{
    rngs::SmallRng,
    RngCore, SeedableRng,
};
use serde::Deserialize;
use thiserror::Error;
use uuid::{
    fmt::Simple,
    Uuid,
};

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .default_service(web::to(HttpResponse::NotFound))
        .service(auth)
        .service(version)
        .service(limits)
        .service(motd)
        .service(id)
        .service(token)

        .service(socket);
}

#[actix_web::get("/api")]
pub async fn auth(web::Header(..): web::Header<UserAgent>, web::Header(Token(..)): web::Header<Token>) -> impl Responder {
    "Hello from `GlennFolker/figura-backend`!"
}

#[actix_web::get("/api/limits")]
pub async fn limits(web::Header(..): web::Header<UserAgent>, web::Header(Token(..)): web::Header<Token>) -> impl Responder {
    serde_json::to_string(&serde_json::json!({
        "rate": {
            "upload": 128,
            "download": 128
        },
        "limits": {
            "maxAvatarSize": 16 * 1024 * 1024
        }
    }))
}

#[actix_web::get("/api/version")]
pub async fn version(web::Header(..): web::Header<UserAgent>, web::Header(Token(..)): web::Header<Token>) -> impl Responder {
    serde_json::to_string(&serde_json::json!({
        "release": "0.1.4",
        "prerelease": "0.1.4",
    }))
}

#[actix_web::get("/api/motd")]
pub async fn motd(web::Header(..): web::Header<UserAgent>, web::Header(Token(..)): web::Header<Token>) -> impl Responder {
    "I am inside your walls. :^)"
}

#[actix_web::get("/ws")]
pub async fn socket(req: HttpRequest, stream: web::Payload) -> impl Responder {
    Ws::start(&req, stream)
}
