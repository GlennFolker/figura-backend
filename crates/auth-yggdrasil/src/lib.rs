use std::time::Duration;

use figura_api::{
    actix::spawn,
    actix_web::{
        rt::task::JoinHandle,
        web,
        HttpRequest,
    },
    anyhow,
    awc::http::StatusCode,
    encode_uuid,
    service::{
        auth::{
            Auth,
            AuthService,
        },
        http::HttpService,
        ServiceLocator,
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
    fn config(&self, locator: &mut dyn ServiceLocator) {
        locator.locate::<AuthService>().add(YggdrasilAuth {
            session_server: self.session_server.clone(),
            timeout: self.timeout,
        });
    }
}

pub struct YggdrasilAuth {
    session_server: String,
    timeout: Duration,
}

impl Auth for YggdrasilAuth {
    fn authenticate(&self, req: &HttpRequest, username: &str, server_id: Uuid) -> JoinHandle<anyhow::Result<Option<Uuid>>> {
        let &Self {
            ref session_server,
            timeout,
        } = self;

        let http = req
            .app_data::<web::Data<HttpService>>()
            .expect("`HttpService` not found")
            .client()
            .clone();

        let session_server = session_server.clone();
        let username = username.to_string();

        spawn(async move {
            let mut response = http
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
