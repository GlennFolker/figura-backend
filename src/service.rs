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

pub struct UserAgent {
    pub name: String,
    pub version: String,
}

#[derive(Error, Debug)]
#[error("invalid `user-agent` string: expected `{{name}}/{{version}}`")]
pub struct UserAgentParseError;
impl header::TryIntoHeaderValue for UserAgent {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        let mut agent = self.name;
        agent.push_str(&self.version);

        header::HeaderValue::from_str(&agent)
    }
}

impl header::Header for UserAgent {
    fn name() -> header::HeaderName {
        header::USER_AGENT
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
        header::from_one_raw_str(msg.headers().get(Self::name()))
    }
}

impl FromStr for UserAgent {
    type Err = UserAgentParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slash = s.find('/').ok_or(UserAgentParseError)?;
        if slash == 0 || slash == s.len() - 1 {
            Err(UserAgentParseError)
        } else {
            Ok(Self {
                name: s[0..slash].to_string(),
                version: s[slash + 1..].to_string(),
            })
        }
    }
}

pub struct Token(pub String);
impl header::TryIntoHeaderValue for Token {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        header::HeaderValue::from_str(&self.0)
    }
}

impl header::Header for Token {
    fn name() -> header::HeaderName {
        header::HeaderName::from_static("token")
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
        header::from_one_raw_str(msg.headers().get(Self::name()))
    }
}

impl FromStr for Token {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[derive(Deserialize)]
pub struct AuthServerId {
    pub username: String,
}

#[derive(Deserialize)]
pub struct AuthToken {
    pub id: String,
}

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

#[actix_web::get("/api/version")]
pub async fn version() -> impl Responder {
    serde_json::to_string(&serde_json::json!({
        "release": "0.1.4",
        "prerelease": "0.1.4",
    }))
}

#[actix_web::get("/api/motd")]
pub async fn motd() -> impl Responder {
    "I am inside your walls. :^)"
}

#[actix_web::get("/api/auth/id")]
pub async fn id(web::Query(query): web::Query<AuthServerId>) -> impl Responder {
    let mut hash = FxHasher::default();
    hash.write(query.username.as_bytes());

    let mut rng = SmallRng::seed_from_u64(hash.finish());
    let mut uuid = [0; 16];
    rng.fill_bytes(&mut uuid);

    Uuid::from_bytes_le(uuid)
        .as_simple().encode_lower(&mut [0; Simple::LENGTH])
        .to_owned()
}

#[actix_web::get("/api/auth/verify")]
pub async fn token(web::Query(query): web::Query<AuthToken>) -> impl Responder {
    query.id
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

#[actix_web::get("/ws")]
pub async fn socket(req: HttpRequest, stream: web::Payload) -> impl Responder {
    Ws::start(&req, stream)
}
