pub use actix;
pub use actix_web;
pub use actix_web_actors;
pub use anyhow;
pub use awc;
pub use log;
pub use once_cell;
pub use rand;
pub use rustls;
pub use rustls_pemfile;
pub use serde;
pub use serde_json;
pub use thiserror;
pub use uuid;

pub mod endpoint;
pub mod service;

use std::{
    self,
    io::BufRead,
    net::SocketAddr,
};

use actix_web::{
    middleware::{
        Logger,
        NormalizePath,
    },
    web,
    App,
    HttpServer,
};
use rustls::{
    pki_types::PrivateKeyDer,
    ClientConfig,
    RootCertStore,
    ServerConfig,
};
use rustls_pemfile::{
    certs,
    pkcs8_private_keys,
};
use uuid::{
    fmt::Simple,
    Uuid,
};

use crate::service::{
    auth::AuthService,
    http::HttpService,
};

#[inline]
pub fn encode_uuid(uuid: Uuid) -> String {
    uuid.as_simple().encode_lower(&mut [0; Simple::LENGTH]).to_string()
}

pub struct Backend<Key: AsReader, Cert: AsReader> {
    port: u16,
    key: Key,
    cert: Cert,
    configs: Vec<Box<dyn BackendConfig>>,
}

impl<Key: AsReader, Cert: AsReader> Backend<Key, Cert> {
    #[inline]
    pub fn new(port: u16, key: Key, cert: Cert) -> Self {
        Self {
            port,
            key,
            cert,
            configs: Vec::new(),
        }
    }

    #[inline]
    pub fn config(mut self, config: impl BackendConfig) -> Self {
        self.configs.push(Box::new(config));
        self
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let certs = certs(self.cert.as_reader()).filter_map(Result::ok).collect();

        let Some(key) = pkcs8_private_keys(self.key.as_reader())
            .filter_map(|key| match key {
                Ok(key) => Some(key),
                Err(e) => {
                    log::warn!("PKCS#8 key error: {e}. Skipping.");
                    None
                }
            })
            .next()
        else {
            anyhow::bail!("Couldn't locate PKCS#8 private keys.");
        };

        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, PrivateKeyDer::from(key))?;

        let client_config = ClientConfig::builder()
            .with_root_certificates({
                let mut store = RootCertStore::empty();
                store.add_parsable_certificates(rustls_native_certs::load_native_certs()?);
                store
            })
            .with_no_client_auth();

        let auth = AuthService::new()?.global()?;
        let http = HttpService::new(client_config)?.global()?;

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let server = HttpServer::new(move || {
            App::new()
                .wrap(NormalizePath::trim())
                .wrap(Logger::default())
                .app_data(web::Data::from(auth.clone()))
                .app_data(web::Data::from(http.clone()))
                .configure(endpoint::config)
        });

        for config in self.configs {
            config.config();
        }

        log::info!("Listening to `{addr}`...");
        Ok(server.bind_rustls_0_23(addr, server_config)?.run().await?)
    }
}

pub trait AsReader {
    fn as_reader(&mut self) -> &mut dyn BufRead;
}

impl<T: BufRead> AsReader for T {
    #[inline]
    fn as_reader(&mut self) -> &mut dyn BufRead {
        self
    }
}

pub trait BackendConfig: 'static {
    fn config(self: Box<Self>);
}
