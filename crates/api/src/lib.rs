pub use actix;
pub use actix_web;
pub use actix_web_actors;
pub use anyhow;
pub use awc;
pub use log;
pub use tokio;
pub use uuid;

pub mod endpoint;
pub mod service;

use std::{
    self,
    any::{
        Any,
        TypeId,
    },
    io::BufRead,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
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
use rand::{
    thread_rng,
    RngCore,
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
    fmt::Hyphenated,
    Builder,
    Uuid,
};

use crate::service::{
    auth::AuthService,
    http::HttpService,
    ServiceLocator,
};

#[inline]
pub fn random_uuid() -> Uuid {
    let mut rng = thread_rng();
    let mut bytes = [0; 16];
    rng.fill_bytes(&mut bytes);

    Builder::from_random_bytes(bytes).into_uuid()
}

#[inline]
pub fn encode_uuid(uuid: Uuid) -> String {
    uuid.as_hyphenated().encode_lower(&mut [0; Hyphenated::LENGTH]).to_string()
}

pub struct Backend<Key: AsReader, Cert: AsReader> {
    pub port: u16,
    pub key: Key,
    pub cert: Cert,

    pub server_id_timeout: Duration,
    pub access_timeout: Duration,

    pub configs: Vec<Box<dyn BackendConfig>>,
}

impl<Key: AsReader, Cert: AsReader> Backend<Key, Cert> {
    pub async fn run(self) -> anyhow::Result<()> {
        let Self {
            port,
            mut key,
            mut cert,
            server_id_timeout,
            access_timeout,
            configs,
        } = self;

        let certs = certs(cert.as_reader()).filter_map(Result::ok).collect();

        let Some(key) = pkcs8_private_keys(key.as_reader())
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

        let configs = Arc::new(configs);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let server = HttpServer::new(move || {
            let client_config = ClientConfig::builder()
                .with_root_certificates({
                    let mut store = RootCertStore::empty();
                    match rustls_native_certs::load_native_certs() {
                        Ok(certs) => {
                            store.add_parsable_certificates(certs);
                        }
                        Err(e) => log::error!("couldn't read native certificate roots: {e}"),
                    }

                    store
                })
                .with_no_client_auth();

            struct Locator {
                auth: AuthService,
                http: HttpService,
            }

            impl ServiceLocator for Locator {
                #[inline]
                fn locate_dyn(&mut self, id: TypeId) -> anyhow::Result<&mut dyn Any> {
                    if id == TypeId::of::<AuthService>() {
                        Ok(&mut self.auth)
                    } else if id == TypeId::of::<HttpService>() {
                        Ok(&mut self.http)
                    } else {
                        anyhow::bail!("invalid service")
                    }
                }
            }

            let mut locator = Locator {
                auth: AuthService::new(server_id_timeout, access_timeout),
                http: HttpService::new(client_config),
            };

            for config in &*configs {
                config.config(&mut locator);
            }

            App::new()
                .wrap(NormalizePath::trim())
                .wrap(Logger::default())
                .app_data(web::Data::new(locator.auth))
                .app_data(web::Data::new(locator.http))
                .configure(endpoint::config)
        });

        log::info!("Listening to `{addr}`...");

        let server = server.bind_rustls_0_23(addr, server_config)?.run();
        server.await?;

        Ok(())
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

pub trait BackendConfig: 'static + Send + Sync {
    fn config(&self, locator: &mut dyn ServiceLocator);
}
