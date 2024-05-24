pub mod auth;
pub mod header;

use actix_web::{
    web,
    HttpResponse,
};

pub fn config(config: &mut web::ServiceConfig) {
    config
        .default_service(web::to(HttpResponse::NotFound))
        .service(auth::get_server_id)
        .service(auth::get_access_token);
}
