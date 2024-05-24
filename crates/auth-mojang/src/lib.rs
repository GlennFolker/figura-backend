use figura_api::{
    anyhow,
    log,
    service::auth::{
        Auth,
        AuthFuture,
        AuthService,
    },
    uuid::Uuid,
    BackendConfig,
};

pub struct MojangAuthConfig {
    pub session_server: String,
}

impl BackendConfig for MojangAuthConfig {
    #[inline]
    fn config(self: Box<Self>) {
        let Self { session_server } = *self;

        log::info!("Mojang session server: https://{}", session_server);
    }
}
