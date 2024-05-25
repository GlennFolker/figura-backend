use std::{
    sync::Arc,
    time::Duration,
};

use figura_api::{
    anyhow,
    encode_uuid,
    log,
    service::{
        auth::{
            Auth,
            AuthFuture,
            AuthService,
        },
        http::HttpService,
    },
    uuid::{
        fmt::Simple,
        Uuid,
    },
    BackendConfig,
};
use figura_api::awc::http::StatusCode;
use figura_api::serde_json::Value;

pub struct YggdrasilConfig {
    pub session_server: String,
}

impl BackendConfig for YggdrasilConfig {
    #[inline]
    fn config(self: Box<Self>) {
        let Self { session_server } = *self;
        log::info!("Authenticating on `{}`.", session_server);

        AuthService::get().unwrap().add(YggdrasilAuth {
            session_server,
            http: HttpService::get().unwrap().clone(),
        });
    }
}

pub struct YggdrasilAuth {
    session_server: String,
    http: Arc<HttpService>,
}

impl Auth for YggdrasilAuth {
    fn authenticate(&self, username: &str, server_id: Uuid) -> AuthFuture<anyhow::Result<bool>> {
        let username = username.to_string();
        let session_server = self.session_server.clone();
        let http = self.http.clone();

        Box::pin(async move {
            let mut response = http
                .client()
                .get(format!(
                    "{}/hasJoined?username={}&serverId={}",
                    session_server,
                    username,
                    encode_uuid(server_id)
                ))
                .timeout(Duration::from_secs(30))
                .send()
                .await?;
            
            if response.status() == StatusCode::OK {
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }
}
