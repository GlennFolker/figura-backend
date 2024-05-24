mod auth;
mod headers;
pub use auth::*;
pub use headers::*;

use actix_web::{
    web,
    HttpResponse,
};

pub fn configure_api(config: &mut web::ServiceConfig) {
    config
        .default_service(web::to(HttpResponse::NotFound))
        .service(get_id)
        .service(get_token);
}
