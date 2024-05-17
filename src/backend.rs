use std::{
    io::BufRead,
    net::SocketAddr,
};
use actix_web::{
    middleware::Logger,
    App, HttpServer,
};
use rustls::{
    pki_types::PrivateKeyDer,
    ServerConfig,
};
use rustls_pemfile::{
    certs, pkcs8_private_keys,
};

pub struct Backend<TlsKey, TlsCert> where
    TlsKey: BufRead,
    TlsCert: BufRead,
{
    port: u16,
    tls_config: (TlsKey, TlsCert),
}

impl<TlsKey, TlsCert> Backend<TlsKey, TlsCert> where
    TlsKey: BufRead,
    TlsCert: BufRead,
{
    pub fn new(port: u16, key: TlsKey, cert: TlsCert) -> Self {
        Self {
            port,
            tls_config: (key, cert),
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let (mut key, mut cert) = self.tls_config;
        let certs = certs(&mut cert).filter_map(Result::ok).collect();
        let Some(key) = pkcs8_private_keys(&mut key).filter_map(Result::ok).next() else {
            anyhow::bail!("Couldn't locate PKCS 8 private keys. Make sure the file path is correct and that it is a valid RSA private key file.");
        };

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, PrivateKeyDer::from(key))?;

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let server = HttpServer::new(|| App::new()
            .wrap(Logger::default())
        );

        Ok(server.bind_rustls_0_22(addr, config)?.run().await?)
    }
}
