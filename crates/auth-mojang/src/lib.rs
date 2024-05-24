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
        AuthService::get().add(MojangAuth {
            session_server: session_server,
        });
    }
}

pub struct MojangAuth {
    session_server: String,
}

impl Auth for MojangAuth {
    fn get_server_id(&self, username: &str) -> AuthFuture<anyhow::Result<Option<Uuid>>> {
        let username = username.to_string();
        Box::pin(async move { Ok(None) })
    }
}
