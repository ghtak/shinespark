pub mod pg;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

pub struct Message<T> {
    pub id: Uuid,
    pub topic: String,
    pub payload: T,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait MessageQueue {
    async fn ack(&self, id: Uuid) -> crate::Result<()>;
    async fn nack(&self, id: Uuid) -> crate::Result<()>;
}

#[async_trait]
pub trait Publisher<T: Serialize + Send + Sync + 'static> {
    async fn publish(&self, topic: &str, payload: T) -> crate::Result<Uuid>;
}

#[async_trait]
pub trait Consumer<T: DeserializeOwned + Send + Sync + 'static> {
    async fn poll(&self, topic: &str) -> crate::Result<Option<Message<T>>>;
}
