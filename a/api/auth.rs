use std::future::Future;
use std::pin::Pin;
use actix_web::{
    web,
    HttpResponse, Responder,
};
use serde::Deserialize;
use uuid::fmt::Simple;
use uuid::Uuid;

pub type BoxFut<Output> = Pin<Box<dyn Future<Output = Output>>>;

pub struct AuthService {
    auths: Vec<Box<dyn Auth>>,
}

impl AuthService {
    pub const fn new() -> Self {
        Self {
            auths: Vec::new(),
        }
    }

    pub fn with(mut self, auth: impl Auth) -> Self {
        self.auths.push(Box::new(auth));
        self
    }

    pub async fn get_id(&self, name: &str) -> Option<Uuid> {
        for prov in &self.auths {
            let id = prov.get_id(name).await;
            if id.is_some() {
                return id
            }
        }

        None
    }
}

pub trait Auth: 'static {
    fn get_id(&self, name: &str) -> BoxFut<Option<Uuid>>;
}

#[derive(Deserialize)]
pub struct GetId {
    pub username: String,
}

#[derive(Deserialize)]
pub struct GetToken {
    pub id: String,
}

#[actix_web::get("/api/auth/id")]
pub async fn get_id(
    web::Query(GetId { username }): web::Query<GetId>,
    web::Data(auth): web::Data<AuthService>,
) -> impl Responder {
    auth.get_id(&username).await
        .map_or_else(
            || HttpResponse::NotFound().body(format!("UUID with username {username} not found")),
            |id| HttpResponse::Ok().body(id.as_simple().encode_lower(&mut [0; Simple::LENGTH]).to_owned()),
        )
}

#[actix_web::get("/api/auth/verify")]
pub async fn get_token(
    web::Query(GetToken { id }): web::Query<GetToken>,
    web::Data(auth): web::Data<AuthService>,
) -> impl Responder {
    id
}
