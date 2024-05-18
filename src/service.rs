use std::hash::Hasher;
use actix::{
    Actor, StreamHandler,
};
use actix_web::{
    web::{
        self,
        ServiceConfig,
    },
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws::{
    self,
    WebsocketContext,
    Message, ProtocolError,
};
use fxhash::FxHasher;
use rand::{
    rngs::SmallRng,
    RngCore, SeedableRng,
};
use serde::Deserialize;
use uuid::{
    fmt::Hyphenated,
    Uuid,
};

#[derive(Deserialize)]
pub struct AuthServerId {
    pub username: String,
}

#[derive(Deserialize)]
pub struct AuthToken {
    pub id: String,
}

pub struct Ws;
impl Actor for Ws {
    type Context = WebsocketContext<Self>;
}

impl StreamHandler<Result<Message, ProtocolError>> for Ws {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {

    }
}

pub fn configure(config: &mut ServiceConfig) {
    config
        .default_service(web::to(HttpResponse::NotFound))
        .service(api)
        .service(version)
        .service(id)
        .service(token)

        .service(web_socket);
}

#[actix_web::get("/api")]
pub async fn api() -> impl Responder {
    "Hello from `GlennFolker/figura-backend`!"
}

#[actix_web::get("/api/version")]
pub async fn version() -> impl Responder {
    "{\
        \"release\":\"2.7.1\",\
        \"prerelease\":\"2.7.1\"\
    }"
}

#[actix_web::get("/api/limits")]
pub async fn limits() -> impl Responder {
    format!("{{\
        \"rate\":{{\
            \"upload\":{},\
            \"download\":{}\
        }},\
        \"limits\":{{\
            \"maxAvatarSize\": {}\
        }}\
    }}", 128, 128, 16 * 1024 * 1024)
}

#[actix_web::get("/api/auth/id")]
pub async fn id(web::Query(query): web::Query<AuthServerId>) -> impl Responder {
    let mut hash = FxHasher::default();
    hash.write(query.username.as_bytes());

    let mut rng = SmallRng::seed_from_u64(hash.finish());
    let mut uuid = [0; 16];
    rng.fill_bytes(&mut uuid);

    Uuid::from_bytes_le(uuid)
        .as_hyphenated().encode_lower(&mut [0; Hyphenated::LENGTH])
        .to_owned()
}

#[actix_web::get("/api/auth/verify")]
pub async fn token(web::Query(query): web::Query<AuthToken>) -> impl Responder {
    query.id
}

#[actix_web::get("/ws")]
pub async fn web_socket(req: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(Ws, &req, stream)
}
