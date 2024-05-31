use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use actix_web::rt::{
    spawn,
    time::{
        sleep,
        Instant,
    },
};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::{
    encode_uuid,
    random_uuid,
};

static INSTANCE: OnceCell<Arc<AuthService>> = OnceCell::new();

pub struct AuthService {
    auths: RwLock<Vec<Arc<dyn Auth>>>,
    server_ids: Arc<RwLock<HashMap<Uuid, (Instant, String)>>>,
    access_tokens: Arc<RwLock<HashMap<Uuid, (Instant, Uuid, Uuid, String, Arc<dyn Auth>)>>>,
}

impl AuthService {
    #[inline]
    pub fn new(server_id_timeout: Duration, access_timeout: Duration) -> anyhow::Result<Self> {
        let auths = RwLock::new(Vec::new());
        let server_ids = Arc::new(RwLock::new(HashMap::new()));
        let access_tokens = Arc::new(RwLock::new(HashMap::new()));

        {
            let server_ids = Arc::downgrade(&server_ids);
            let access_tokens = Arc::downgrade(&access_tokens);

            spawn(async move {
                // If either are deallocated, assume the server is dying anyway so don't bother.
                while let (Some(server_ids), Some(access_tokens)) = (server_ids.upgrade(), access_tokens.upgrade()) {
                    let now = Instant::now();
                    server_ids.write().retain(|&id, (time, name)| {
                        if now - *time >= server_id_timeout {
                            log::warn!("Unassigning {} from {name} due to timeout!", encode_uuid(id));
                            false
                        } else {
                            true
                        }
                    });

                    access_tokens
                        .write()
                        .retain(|&id, (time, _server_id, _user_id, name, _auth)| {
                            if now - *time >= access_timeout {
                                log::warn!("Invalidating {} from {name} due to timeout!", encode_uuid(id));
                                false
                            } else {
                                true
                            }
                        });

                    sleep(Duration::from_secs(1)).await;
                }
            });
        }

        Ok(Self {
            auths,
            server_ids,
            access_tokens,
        })
    }

    #[inline]
    pub(crate) fn global(self) -> anyhow::Result<Arc<Self>> {
        INSTANCE
            .try_insert(Arc::new(self))
            .cloned()
            .map_err(|_| anyhow::anyhow!("`AuthService` already initialized"))
    }

    #[inline]
    pub fn get() -> anyhow::Result<&'static Arc<Self>> {
        INSTANCE.get().ok_or_else(|| anyhow::anyhow!("`AuthService` not initialized"))
    }

    #[inline]
    pub fn add(&self, auth: impl Auth) {
        self.auths.write().push(Arc::new(auth));
    }

    pub fn assign_server_id(&self, username: &str) -> Uuid {
        let server_id = random_uuid();
        self.server_ids
            .write()
            .insert(server_id, (Instant::now(), username.to_string()));
        server_id
    }

    pub async fn obtain_access_token(&self, server_id: Uuid) -> Option<Uuid> {
        let mut server_ids = self.server_ids.write();
        let (.., name) = server_ids.remove(&server_id)?;

        for auth in &*self.auths.read() {
            match auth.authenticate(&name, server_id).await {
                Ok(Some(user_id)) => {
                    let token = random_uuid();
                    self.access_tokens
                        .write()
                        .insert(token, (Instant::now(), server_id, user_id, name, auth.clone()));
                    return Some(token)
                }
                Ok(None) => continue,
                Err(e) => log::error!("Couldn't authenticate {name}: {e}"),
            }
        }

        None
    }

    pub fn check_access_token(&self, access_token: Uuid) -> bool {
        self.access_tokens.read().contains_key(&access_token)
    }

    pub async fn refresh_access_token(&self, access_token: Uuid) -> bool {
        let mut access_tokens = self.access_tokens.write();
        let Some((time, server_id, _user_id, name, auth)) = access_tokens.get_mut(&access_token) else {
            return false
        };

        if auth.authenticate(name, *server_id).await.unwrap_or_else(|e| {
            log::error!("Couldn't check authenticity of {name}: {e}");
            None
        }).is_some() {
            *time = Instant::now();
            true
        } else {
            false
        }
    }
}

pub type AuthFuture<Output> = Pin<Box<dyn Future<Output = Output> + 'static>>;

pub trait Auth: 'static + Send + Sync {
    fn authenticate(&self, username: &str, server_id: Uuid) -> AuthFuture<anyhow::Result<Option<Uuid>>>;
}
