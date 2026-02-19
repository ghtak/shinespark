use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub uid: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i32,
    pub code: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum AuthProvider {
    Local,
    Google,
    Kakao,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserIdentity {
    pub id: i64,
    pub user_id: i64,
    pub provider: AuthProvider,
    pub provider_user_id: String,
    pub credential_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}
