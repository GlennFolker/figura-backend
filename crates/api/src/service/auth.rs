use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        RwLock,
    },
};

use once_cell::sync::OnceCell;
use rand::{
    thread_rng,
    RngCore,
};
use uuid::Uuid;

static INSTANCE: OnceCell<Arc<AuthService>> = OnceCell::new();

pub struct AuthService {
    auths: RwLock<Vec<Box<dyn Auth>>>,
    server_ids: RwLock<HashMap<Uuid, String>>,
}

impl AuthService {
    pub const MAX_USERNAME_LEN: usize = 64;

    #[inline]
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            auths: RwLock::new(Vec::new()),
            server_ids: RwLock::new(HashMap::new()),
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
        self.auths.write().expect("couldn't write-lock").push(Box::new(auth));
    }

    pub fn assign_server_id(&self, username: &str) -> Uuid {
        let mut rng = thread_rng();
        let mut bytes = [0; 16];
        rng.fill_bytes(&mut bytes);

        let id = Uuid::from_bytes(bytes);
        self.server_ids
            .write()
            .expect("couldn't write-lock")
            .insert(id, username.to_string());
        id
    }

    pub async fn authenticate(&self, server_id: Uuid) -> anyhow::Result<()> {
        let server_ids = self.server_ids.read().expect("couldn't read-lock");
        let name = server_ids
            .get(&server_id)
            .ok_or_else(|| anyhow::anyhow!("invalid server ID"))?;
        
        let mut errors = Vec::new();
        for auth in &*self.auths.read().expect("couldn't read-lock") {
            match auth.authenticate(&name, server_id).await {
                Ok(true) => return Ok(()),
                Ok(false) => continue,
                Err(e) => errors.push(e),
            }
        }
        
        Err(anyhow::anyhow!("{errors:?}"))
    }
}

pub type AuthFuture<Output> = Pin<Box<dyn Future<Output = Output> + 'static>>;

pub trait Auth: Send + Sync + 'static {
    fn authenticate(&self, username: &str, server_id: Uuid) -> AuthFuture<anyhow::Result<bool>>;
}
