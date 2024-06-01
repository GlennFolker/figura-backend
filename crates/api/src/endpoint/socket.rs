use actix_web::{
    web,
    HttpRequest,
    Responder,
};

use crate::{
    service::auth::AuthService,
    socket::actor::Socket,
};

#[actix_web::get("/ws")]
pub async fn web_socket(req: HttpRequest, stream: web::Payload, auth: web::Data<AuthService>) -> impl Responder {
    Socket::start(auth.into_inner(), &req, stream)
}
