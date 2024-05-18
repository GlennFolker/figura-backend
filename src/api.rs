use actix_web::{
    web::{
        self,
        ServiceConfig,
    },
    HttpResponse, Responder,
};

pub fn configure(config: &mut ServiceConfig) {
    config
        .default_service(web::to(HttpResponse::NotFound))
        .service(api);
}

#[actix_web::get("/api")]
pub async fn api() -> impl Responder {
    "Hello from `GlennFolker/figura-backend`!"
}
