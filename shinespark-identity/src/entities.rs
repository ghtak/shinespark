use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "text", rename_all = "lowercase")] // DB 타입이 문자열임을 명시
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Pending,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "text", rename_all = "lowercase")] // DB 타입이 문자열임을 명시
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    Local,
    Google,
    Apple,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "text", rename_all = "lowercase")] // DB 타입이 문자열임을 명시
#[serde(rename_all = "lowercase")]
pub enum UserAction {
    Login,
    Logout,
    StatusChanged,
    CredentialUpdated,
    ProfileUpdated,
}

// 시스템의 핵심 식별 주체인 사용자 정보입니다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,            // 데이터베이스 내부 식별용 PK (Auto Increment)
    pub uid: Uuid,          // 외부 노출용 고유 식별자 (API 통신, 토큰 발급 등에 사용)
    pub name: String,       // 사용자 이름 또는 닉네임
    pub email: String,      // 연락 및 주요 인증 기준이 되는 이메일 주소
    pub status: UserStatus, // 계정의 현재 활성화 상태

    pub created_at: DateTime<Utc>, // 레코드 최초 생성 일시
    pub updated_at: DateTime<Utc>, // 레코드 최종 수정 일시
}

impl User {
    pub fn new(name: String, email: String, status: UserStatus) -> Self {
        Self {
            id: 0,
            uid: uuid::Uuid::new_v4(),
            name,
            email,
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// 사용자의 인증 수단 및 자격 증명(Credential) 정보입니다. (다중 플랫폼 로그인 지원)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserIdentity {
    pub id: i64,                         // 데이터베이스 내부 식별용 PK
    pub user_id: i64,                    // 연관된 `User`의 PK (FK)
    pub provider: AuthProvider,          // 해당 인증의 제공자 (Local, Google, Apple 등)
    pub provider_uid: String, // 인증 제공자 측의 고유 식별자 (소셜 로그인의 경우 해당 플랫폼의 사용자 ID)
    pub credential_hash: Option<String>, // (Local 인증 전용) 암호화된 비밀번호 해시값. 소셜 로그인 등 비밀번호가 없는 경우 None.
    pub created_at: DateTime<Utc>,       // 연동 정보 등록 일시
    pub updated_at: DateTime<Utc>,       // 연동 정보 상태 변경 일시
}

impl UserIdentity {
    pub fn new(
        user_id: i64,
        provider: AuthProvider,
        provider_uid: String,
        credential_hash: Option<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id,
            provider,
            provider_uid,
            credential_hash,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// 사용자와 관련된 핵심 액션(로그인, 상태 변경 등)의 이력을 남기는 감사 로그입니다.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAuditLog {
    pub id: i64,                     // 데이터베이스 내부 식별용 PK
    pub user_id: i64,                // 액션을 수행한(혹은 대상이 된) 사용자의 PK
    pub action: UserAction,          // 수행된 액션의 카테고리
    pub description: Option<String>, // 액션에 대한 상세 부가 정보 (필요시 어떤 필드가 어떻게 바뀌었는지 문자열이나 JSON 기록)
    pub ip_address: Option<String>,  // 요청을 보낸 사용자의 접속 IP 주소
    pub user_agent: Option<String>,  // 접속 기기 및 브라우저 정보 (User-Agent)
    pub is_success: bool,            // 액션의 최종 성공 여부 (예: 로그인 실패 이력 관리용)
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: i64,
    pub code: String, // dot 으로 구분된 권한 코드 Resource.action.scope
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String, // 역할 이름 (예: "admin", "user")
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub id: i64,
    pub role_id: i64,
    pub permission_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRole {
    pub id: i64,
    pub user_id: i64,
    pub role_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserWithRoles {
    pub user: User,
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserWithIdentities {
    pub user: User,
    pub identities: Vec<UserIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAggregate {
    pub user: User,
    pub role_ids: Vec<i64>,
    pub identities: Vec<UserIdentity>,
}

impl UserStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Pending => "pending",
            Self::Suspended => "suspended",
            Self::Deleted => "deleted",
        }
    }
}

impl TryFrom<String> for UserStatus {
    type Error = shinespark::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "pending" => Ok(Self::Pending),
            "suspended" => Ok(Self::Suspended),
            "deleted" => Ok(Self::Deleted),
            _ => Err(shinespark::Error::IllegalState(
                format!("Invalid user status: {}", value).into(),
            )),
        }
    }
}

impl AuthProvider {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Local => "local",
            Self::Google => "google",
            Self::Apple => "apple",
        }
    }
}

impl TryFrom<String> for AuthProvider {
    type Error = shinespark::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Self::Local),
            "google" => Ok(Self::Google),
            "apple" => Ok(Self::Apple),
            _ => Err(shinespark::Error::IllegalState(
                format!("Invalid auth provider: {}", value).into(),
            )),
        }
    }
}

impl UserAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Login => "login",
            Self::Logout => "logout",
            Self::StatusChanged => "status_changed",
            Self::CredentialUpdated => "credential_updated",
            Self::ProfileUpdated => "profile_updated",
        }
    }
}

impl TryFrom<String> for UserAction {
    type Error = shinespark::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "login" => Ok(Self::Login),
            "logout" => Ok(Self::Logout),
            "status_changed" => Ok(Self::StatusChanged),
            "credential_updated" => Ok(Self::CredentialUpdated),
            "profile_updated" => Ok(Self::ProfileUpdated),
            _ => Err(shinespark::Error::IllegalState(
                format!("Invalid user action: {}", value).into(),
            )),
        }
    }
}
