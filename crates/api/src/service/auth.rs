use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        RwLock,
    },
};

use once_cell::sync::Lazy;
use uuid::Uuid;

pub struct AuthService {
    auths: RwLock<Vec<Box<dyn Auth>>>,
}

impl AuthService {
    #[inline]
    pub fn get() -> &'static Arc<Self> {
        static SERVICE: Lazy<Arc<AuthService>> = Lazy::new(|| {
            Arc::new(AuthService {
                auths: RwLock::new(Vec::new()),
            })
        });

        &SERVICE
    }

    #[inline]
    pub fn add(&self, auth: impl Auth) {
        self.auths.write().expect("couldn't write-lock").push(Box::new(auth));
    }

    /*pub async fn get_server_id(&self, username: &str) -> anyhow::Result<Option<Uuid>> {
        for auth in &*self.auths.read().expect("couldn't read-lock") {
            match auth.get_server_id(username).await {
                // `Ok(None)` means the user failed to authenticate because it's not governed under
                // the authenticator.
                // |> Continue finding an appropriate one.
                Ok(None) => continue,
                // `Ok(Some(...))` means the authenticator successfully grants a server ID.
                // `Err(...)` means the user is governed under the authenticator, but an error
                // occurred while trying to map a server ID.
                // |> Either way, return the result immediately.
                result => return result,
            }
        }

        anyhow::bail!("failed to authenticate a server ID for {username}")
    }*/
}

pub type AuthFuture<Output> = Pin<Box<dyn Future<Output = Output> + 'static>>;

pub trait Auth: Send + Sync + 'static {
    //fn get_server_id(&self, username: &str, server_id: Uuid) -> AuthFuture<anyhow::Result<Option<Uuid>>>;
}
