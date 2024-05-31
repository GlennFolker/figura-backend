pub mod auth;
pub mod header;

use actix_web::{
    web,
    HttpResponse,
};

pub fn config(config: &mut web::ServiceConfig) {
    config
        .default_service(web::to(HttpResponse::NotFound))
        .service(auth::refresh_access_token)
        .service(auth::assign_server_id)
        .service(auth::obtain_access_token);
}
