use shinespark::Result;
use shinespark::db::{AppDbDriver, AppDbHandle};
use sqlx::Pool;

use crate::entity::User;

#[async_trait::async_trait]
pub trait UserService: Sync + Send {
    async fn check_email_exists(&self, email: &str) -> Result<bool>;
}

pub struct DefaultUserService {
    pub pool: Pool<AppDbDriver>,
}

#[async_trait::async_trait]
pub trait UserServiceTx: Sync + Send {
    async fn register_local_user(
        &self,
        h: &mut AppDbHandle<'_>,
        email: &str,
        credential_hash: &str,
        name: Option<&str>,
    ) -> Result<User>;
}
