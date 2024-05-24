use std::sync::{
    Arc,
    RwLock,
};

use once_cell::sync::Lazy;
use rustls::ClientConfig;

pub struct HttpService {
    config: RwLock<Option<ClientConfig>>,
}

impl HttpService {
    #[inline]
    pub fn get() -> &'static Arc<Self> {
        static SERVICE: Lazy<Arc<HttpService>> = Lazy::new(|| {
            Arc::new(HttpService {
                config: RwLock::new(None),
            })
        });
        &SERVICE
    }

    #[inline]
    pub fn config(&self, config: ClientConfig) {
        *self.config.write().expect("couldn't write-lock") = Some(config);
    }
}
