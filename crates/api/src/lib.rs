pub use actix;
pub use actix_web;
pub use actix_web_actors;
pub use anyhow;
pub use awc;
pub use log;
pub use once_cell;
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
    ServerConfig,
};
use rustls_pemfile::{
    certs,
    pkcs8_private_keys,
};

use crate::service::{
    auth::AuthService,
    http::HttpService,
};

pub struct Backend<Key: AsReader, Cert: AsReader> {
    port: u16,
    key: Key,
    cert: Cert,
    configs: Vec<Box<dyn BackendConfig>>,
}

impl<Key: AsReader, Cert: AsReader> Backend<Key, Cert> {
    pub fn new(port: u16, key: Key, cert: Cert) -> Self {
        Self {
            port,
            key,
            cert,
            configs: Vec::new(),
        }
    }

    pub fn config(mut self, config: impl BackendConfig) -> Self {
        self.configs.push(Box::new(config));
        self
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let certs = certs(self.cert.as_reader())
            .filter_map(Result::ok)
            .collect();
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

        for config in self.configs {
            config.config();
        }

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, PrivateKeyDer::from(key))?;

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let server = HttpServer::new(|| {
            App::new()
                .wrap(NormalizePath::trim())
                .wrap(Logger::default())
                .app_data(web::Data::from(AuthService::get().clone()))
                .app_data(web::Data::from(HttpService::get().clone()))
                .configure(endpoint::config)
        });

        log::info!("Listening to `{addr}`...");
        Ok(server.bind_rustls_0_22(addr, config)?.run().await?)
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
