use std::sync::Arc;

use actix_web::http::header::USER_AGENT;
use rustls::ClientConfig;

use crate::service::Service;

pub struct HttpService {
    client: awc::Client,
}

impl Service for HttpService {}

impl HttpService {
    #[inline]
    pub fn new(config: ClientConfig) -> Self {
        Self {
            client: awc::Client::builder()
                .add_default_header((USER_AGENT, "figura-backend/0.1.0"))
                .connector(awc::Connector::new().rustls_0_23(Arc::new(config)))
                .finish(),
        }
    }

    #[inline]
    pub fn client(&self) -> &awc::Client {
        &self.client
    }
}
