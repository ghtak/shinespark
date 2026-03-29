#[async_trait::async_trait]
pub trait UserRepository: Sync + Send {}
