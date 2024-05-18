use std::{
    io::BufRead,
    net::SocketAddr,
};
use actix_web::{
    middleware::Logger,
    web,
    App, HttpServer,
    HttpResponse, Responder,
};
use rustls::{
    pki_types::PrivateKeyDer,
    ServerConfig,
};
use rustls_pemfile::{
    certs, pkcs8_private_keys,
};

pub struct Backend<'k, 'c> {
    port: u16,
    key: &'k mut dyn BufRead,
    cert: &'c mut dyn BufRead,
}

impl<'k, 'c> Backend<'k, 'c> {
    pub fn new(port: u16, key: &'k mut dyn BufRead, cert: &'c mut dyn BufRead) -> Self {
        Self { port, key, cert, }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let certs = certs(self.cert).filter_map(Result::ok).collect();
        let Some(key) = pkcs8_private_keys(self.key).filter_map(|key| match key {
            Ok(key) => Some(key),
            Err(e) => {
                log::warn!("PKCS#8 key error: {e}. Skipping.");
                None
            },
        }).next() else {
            anyhow::bail!("Couldn't locate PKCS#8 private keys.");
        };

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, PrivateKeyDer::from(key))?;

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let server = HttpServer::new(|| App::new()
            .wrap(Logger::default())

            .default_service(web::to(HttpResponse::NotFound))
            .service(api)
        );

        log::info!("Listening to port {}...", self.port);
        Ok(server.bind_rustls_0_22(addr, config)?.run().await?)
    }
}

#[actix_web::get("/api")]
pub async fn api() -> impl Responder {
    "Hello from `GlennFolker/figura-backend`!"
}
