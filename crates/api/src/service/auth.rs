use std::{
    sync::Arc,
    time::Duration,
};

use actix_web::{
    rt::{
        spawn,
        task::JoinHandle,
        time::{
            sleep,
            Instant,
        },
    },
    HttpRequest,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::{
    encode_uuid,
    random_uuid,
    service::Service,
    FxHashMap,
    FxHashSet,
};

struct Token {
    time: RwLock<Instant>,
    server_id: Uuid,
    user_id: Uuid,
    name: String,
    auth: Arc<dyn Auth>,
}

pub struct AuthService {
    auths: Vec<Arc<dyn Auth>>,
    server_ids: Arc<RwLock<FxHashSet<Uuid>>>,
    access_tokens: Arc<RwLock<FxHashSet<Uuid>>>,
    checker: JoinHandle<()>,
}

impl Service for AuthService {}

pub trait Auth: 'static + Send + Sync {
    fn authenticate(&self, req: &HttpRequest, username: &str, server_id: Uuid) -> JoinHandle<anyhow::Result<Option<Uuid>>>;
}

static SERVER_IDS: Lazy<Arc<RwLock<FxHashMap<Uuid, (Instant, String)>>>> = Lazy::new(Default::default);
static ACCESS_TOKENS: Lazy<Arc<RwLock<FxHashMap<Uuid, Arc<Token>>>>> = Lazy::new(Default::default);

impl AuthService {
    #[inline]
    pub fn new(server_id_timeout: Duration, access_timeout: Duration) -> Self {
        let auths = Vec::new();
        let server_ids = Arc::new(RwLock::new(FxHashSet::default()));
        let access_tokens = Arc::new(RwLock::new(FxHashSet::default()));

        let checker = {
            let server_ids = Arc::downgrade(&server_ids);
            let access_tokens = Arc::downgrade(&access_tokens);

            spawn(async move {
                // If either are deallocated, assume the server is dying anyway so don't bother.
                while let (Some(server_ids), Some(access_tokens)) = (server_ids.upgrade(), access_tokens.upgrade()) {
                    let now = Instant::now();

                    {
                        server_ids.write().retain(|&id| {
                            if let Some(&(time, ..)) = { SERVER_IDS.read().get(&id) } {
                                if now - time >= server_id_timeout {
                                    SERVER_IDS.write().remove(&id);
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        });
                    }

                    {
                        access_tokens.write().retain(|&id| {
                            if let Some(token) = { ACCESS_TOKENS.read().get(&id).cloned() } {
                                if now - *token.time.read() >= access_timeout {
                                    ACCESS_TOKENS.write().remove(&id);
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        });
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            })
        };

        Self {
            auths,
            server_ids,
            access_tokens,
            checker,
        }
    }

    #[inline]
    pub fn add(&mut self, auth: impl Auth) {
        self.auths.push(Arc::new(auth));
    }

    pub fn assign_server_id(&self, username: &str) -> Uuid {
        let server_id = random_uuid();
        SERVER_IDS.write().insert(server_id, (Instant::now(), username.to_string()));
        server_id
    }

    pub async fn obtain_access_token(&self, req: &HttpRequest, server_id: Uuid) -> anyhow::Result<Option<Uuid>> {
        let Some((.., name)) = ({ SERVER_IDS.write().remove(&server_id) }) else {
            return Ok(None)
        };

        for auth in &self.auths {
            match auth.authenticate(req, &name, server_id).await? {
                Ok(Some(user_id)) => {
                    let token = random_uuid();
                    ACCESS_TOKENS.write().insert(
                        token,
                        Arc::new(Token {
                            time: RwLock::new(Instant::now()),
                            server_id,
                            user_id,
                            name,
                            auth: auth.clone(),
                        }),
                    );

                    return Ok(Some(token))
                }
                Ok(None) => {}
                Err(e) => log::error!("Couldn't authenticate {name}: {e}"),
            }
        }

        Ok(None)
    }

    pub fn check_access_token(&self, access_token: Uuid) -> bool {
        ACCESS_TOKENS.read().contains_key(&access_token)
    }

    pub async fn refresh_access_token(&self, req: &HttpRequest, access_token: Uuid) -> anyhow::Result<bool> {
        let Some(token) = ({ ACCESS_TOKENS.read().get(&access_token).cloned() }) else {
            return Ok(false)
        };

        if token
            .auth
            .authenticate(req, &token.name, token.server_id)
            .await?
            .unwrap_or_else(|e| {
                log::error!("Couldn't check authenticity of {}: {e}", token.name);
                None
            })
            .is_some()
        {
            *token.time.write() = Instant::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Drop for AuthService {
    fn drop(&mut self) {
        for id in self.server_ids.write().drain() {
            SERVER_IDS.write().remove(&id);
        }

        for id in self.access_tokens.write().drain() {
            ACCESS_TOKENS.write().remove(&id);
        }

        self.checker.abort();
    }
}
