use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::FromRow;
use uuid::Uuid;

use super::{Consumer, Message, MessageQueue, Publisher};
use crate::db::Database;

fn db_err(e: sqlx::Error) -> crate::Error {
    crate::Error::DatabaseError(anyhow::anyhow!(e))
}

pub struct PgMessageQueue {
    pub db: Database,
}

impl PgMessageQueue {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn reap_stale(&self) -> crate::Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE shs_mq_messages
            SET    status     = CASE WHEN attempts >= max_attempts THEN 'failed' ELSE 'pending' END,
                   locked_at  = NULL,
                   updated_at = NOW()
            WHERE  status    = 'processing'
              AND  locked_at < NOW() - INTERVAL '5 minutes'
            "#,
        )
        .execute(&self.db.inner)
        .await
        .map_err(db_err)?;
        Ok(result.rows_affected())
    }
}

#[derive(FromRow)]
struct MqRecord {
    id: Uuid,
    topic: String,
    payload: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl MessageQueue for PgMessageQueue {
    async fn ack(&self, id: Uuid) -> crate::Result<()> {
        sqlx::query(
            "UPDATE shs_mq_messages SET status = 'done', done_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.db.inner)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    async fn nack(&self, id: Uuid) -> crate::Result<()> {
        sqlx::query(
            r#"
            UPDATE shs_mq_messages
            SET    status     = CASE WHEN attempts >= max_attempts THEN 'failed' ELSE 'pending' END,
                   locked_at  = NULL,
                   updated_at = NOW()
            WHERE  id = $1
            "#,
        )
        .bind(id)
        .execute(&self.db.inner)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}

#[async_trait]
impl<T: Serialize + Send + Sync + 'static> Publisher<T> for PgMessageQueue {
    async fn publish(&self, topic: &str, payload: T) -> crate::Result<Uuid> {
        let value = serde_json::to_value(&payload)
            .map_err(|e| crate::Error::Internal(anyhow::anyhow!(e)))?;

        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO shs_mq_messages (topic, payload) VALUES ($1, $2) RETURNING id",
        )
        .bind(topic)
        .bind(value)
        .fetch_one(&self.db.inner)
        .await
        .map_err(db_err)?;

        Ok(id)
    }
}

#[async_trait]
impl<T: DeserializeOwned + Send + Sync + 'static> Consumer<T> for PgMessageQueue {
    async fn poll(&self, topic: &str) -> crate::Result<Option<Message<T>>> {
        let row: Option<MqRecord> = sqlx::query_as(
            r#"
            WITH next AS (
                SELECT id FROM shs_mq_messages
                WHERE  topic    = $1
                  AND  status   = 'pending'
                  AND  attempts < max_attempts
                ORDER BY created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE shs_mq_messages
            SET    status     = 'processing',
                   locked_at  = NOW(),
                   attempts   = attempts + 1,
                   updated_at = NOW()
            FROM   next
            WHERE  shs_mq_messages.id = next.id
            RETURNING shs_mq_messages.id,
                      shs_mq_messages.topic,
                      shs_mq_messages.payload,
                      shs_mq_messages.created_at
            "#,
        )
        .bind(topic)
        .fetch_optional(&self.db.inner)
        .await
        .map_err(db_err)?;

        row.map(|r| {
            let payload = serde_json::from_value(r.payload)
                .map_err(|e| crate::Error::Internal(anyhow::anyhow!(e)))?;
            Ok(Message {
                id: r.id,
                topic: r.topic,
                payload,
                created_at: r.created_at,
            })
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        value: String,
    }

    #[tokio::test]
    #[ignore]
    async fn test_publish_poll_ack() {
        let db = Database::new_dotenv().await.unwrap();
        let mq = PgMessageQueue::new(db);

        let id = mq
            .publish(
                "test-topic",
                TestPayload {
                    value: "hello".into(),
                },
            )
            .await
            .unwrap();

        let msg: Option<Message<TestPayload>> = mq.poll("test-topic").await.unwrap();
        let msg = msg.expect("message should exist");
        assert_eq!(msg.id, id);
        assert_eq!(msg.payload.value, "hello");

        mq.ack(msg.id).await.unwrap();

        let next: Option<Message<TestPayload>> = mq.poll("test-topic").await.unwrap();
        assert!(next.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_nack_retry_to_failed() {
        let db = Database::new_dotenv().await.unwrap();
        let mq = PgMessageQueue::new(db);

        mq.publish(
            "retry-topic",
            TestPayload {
                value: "retry".into(),
            },
        )
        .await
        .unwrap();

        for _ in 0..3 {
            let msg: Message<TestPayload> =
                mq.poll("retry-topic").await.unwrap().expect("should exist");
            mq.nack(msg.id).await.unwrap();
        }

        let next: Option<Message<TestPayload>> = mq.poll("retry-topic").await.unwrap();
        assert!(next.is_none(), "exhausted message must not be polled");
    }
}
