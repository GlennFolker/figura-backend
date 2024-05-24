use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc,
        RwLock,
    },
};

use actix_web::http::header;
use once_cell::{
    sync::Lazy,
    unsync::OnceCell,
};
use rustls::ClientConfig;

pub struct HttpService {
    config: RwLock<Option<Arc<ClientConfig>>>,
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
        *self.config.write().expect("couldn't write-lock") = Some(Arc::new(config));
    }

    pub fn client(&self) -> Rc<awc::Client> {
        thread_local! {
            static CLIENT: OnceCell<Rc<awc::Client>> = const { OnceCell::new() };
        }

        CLIENT.with(|cell| {
            cell.get_or_init(|| {
                let config = self.config.read().expect("couldn't read-lock");
                let config = config.as_ref().cloned().expect("`HttpService` has no client TLS config");

                Rc::new(
                    awc::Client::builder()
                        .add_default_header((header::USER_AGENT, "figura-backend/0.1.0"))
                        .connector(awc::Connector::new().rustls_0_23(config))
                        .finish(),
                )
            })
            .clone()
        })
    }
}
