use std::sync::Arc;

use once_cell::sync::Lazy;

pub struct HttpService {}

impl HttpService {
    #[inline]
    pub fn get() -> &'static Arc<Self> {
        static SERVICE: Lazy<Arc<HttpService>> = Lazy::new(|| Arc::new(HttpService {}));

        &SERVICE
    }
}
