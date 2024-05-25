use std::{
    rc::Rc,
    sync::Arc,
};

use actix_web::http::header::USER_AGENT;
use once_cell::sync::OnceCell;
use rustls::ClientConfig;

static INSTANCE: OnceCell<Arc<HttpService>> = OnceCell::new();

pub struct HttpService {
    config: Arc<ClientConfig>,
}

impl HttpService {
    #[inline]
    pub fn new(config: ClientConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config: Arc::new(config),
        })
    }

    #[inline]
    pub(crate) fn global(self) -> anyhow::Result<Arc<Self>> {
        INSTANCE
            .try_insert(Arc::new(self))
            .cloned()
            .map_err(|_| anyhow::anyhow!("`HttpService` already initialized"))
    }

    #[inline]
    pub fn get() -> anyhow::Result<&'static Arc<Self>> {
        INSTANCE.get().ok_or_else(|| anyhow::anyhow!("`HttpService` not initialized"))
    }

    pub fn client(&self) -> Rc<awc::Client> {
        thread_local! {
            static CLIENT: OnceCell<Rc<awc::Client>> = const { OnceCell::new() };
        }

        CLIENT.with(|cell| {
            cell.get_or_init(|| {
                Rc::new(
                    awc::Client::builder()
                        .add_default_header((USER_AGENT, "figura-backend/0.1.0"))
                        .connector(awc::Connector::new().rustls_0_23(self.config.clone()))
                        .finish(),
                )
            })
            .clone()
        })
    }
}
