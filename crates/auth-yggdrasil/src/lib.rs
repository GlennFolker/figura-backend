use std::{
    sync::Arc,
    time::Duration,
};

use figura_api::{
    anyhow,
    awc::http::StatusCode,
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
    uuid::Uuid,
    BackendConfig,
};

pub struct YggdrasilConfig {
    pub session_server: String,
    pub timeout: Duration,
}

impl BackendConfig for YggdrasilConfig {
    #[inline]
    fn config(self: Box<Self>) {
        let Self { session_server, timeout } = *self;
        log::info!("Authenticating on `{session_server}`.");

        AuthService::get().unwrap().add(YggdrasilAuth {
            session_server,
            timeout,
            http: HttpService::get().unwrap().clone(),
        });
    }
}

pub struct YggdrasilAuth {
    session_server: String,
    timeout: Duration,
    http: Arc<HttpService>,
}

impl Auth for YggdrasilAuth {
    fn authenticate(&self, username: &str, server_id: Uuid) -> AuthFuture<anyhow::Result<Option<Uuid>>> {
        let username = username.to_string();
        let session_server = self.session_server.clone();
        let timeout = self.timeout;
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
                .timeout(timeout)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("couldn't send HTTP GET request: {e}"))?;

            if response.status() == StatusCode::OK {
                let response = response.json::<serde_json::Value>().await?;
                Ok(Some(
                    response
                        .get("id")
                        .ok_or_else(|| anyhow::anyhow!("`id` field not found in response"))?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("`id` field is not a string"))
                        .and_then(|value| Uuid::try_parse(value).map_err(anyhow::Error::from))?,
                ))
            } else {
                Ok(None)
            }
        })
    }
}
