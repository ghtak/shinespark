use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// 사용자의 현재 상태를 나타냅니다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")] // DB 관례에 따라 소문자 사용 (예: "local", "google")
pub enum Status {
    Active,    // 정상적으로 활동 중인 상태
    Inactive,  // 비활성화된 상태 (휴면 등)
    Pending,   // 가입 후 이메일 인증 등 대기 상태
    Suspended, // 관리자에 의해 이용 정지된 상태
    Deleted,   // 탈퇴 처리된 상태
}

// 시스템에 연동된 인증 제공자 종류를 나타냅니다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")] // DB 관례에 따라 소문자 사용 (예: "local", "google")
pub enum AuthProvider {
    Local,  // 자체 회원가입 (이메일/비밀번호)
    Google, // 구글 소셜 로그인
    Apple,  // 애플 소셜 로그인
}

// 시스템의 핵심 식별 주체인 사용자 정보입니다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u64,        // 데이터베이스 내부 식별용 PK (Auto Increment)
    pub uid: Uuid,      // 외부 노출용 고유 식별자 (API 통신, 토큰 발급 등에 사용)
    pub name: String,   // 사용자 이름 또는 닉네임
    pub email: String,  // 연락 및 주요 인증 기준이 되는 이메일 주소
    pub status: Status, // 계정의 현재 활성화 상태

    pub created_at: DateTime<Utc>, // 레코드 최초 생성 일시
    pub updated_at: DateTime<Utc>, // 레코드 최종 수정 일시
}

// 사용자의 인증 수단 및 자격 증명(Credential) 정보입니다. (다중 플랫폼 로그인 지원)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserIdentity {
    pub id: u64,                         // 데이터베이스 내부 식별용 PK
    pub user_id: u64,                    // 연관된 `User`의 PK (FK)
    pub provider: AuthProvider,          // 해당 인증의 제공자 (Local, Google, Apple 등)
    pub provider_uid: String, // 인증 제공자 측의 고유 식별자 (소셜 로그인의 경우 해당 플랫폼의 사용자 ID)
    pub credential_hash: Option<String>, // (Local 인증 전용) 암호화된 비밀번호 해시값. 소셜 로그인 등 비밀번호가 없는 경우 None.
    pub created_at: DateTime<Utc>,       // 연동 정보 등록 일시
    pub updated_at: DateTime<Utc>,       // 연동 정보 상태 변경 일시
}

// 감사 로그(Audit Log)에 기록될 사용자 관련 액션 종류입니다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum UserAction {
    Login,             // 시스템 로그인 시도 (성공/실패 포함)
    Logout,            // 시스템 로그아웃
    StatusChanged,     // 계정 활성화 상태 변경 (예: Pending -> Active)
    CredentialUpdated, // 인증 정보(비밀번호 등) 업데이트
    ProfileUpdated,    // 프로필 정보(이름, 이메일 등) 수정
}

// 사용자와 관련된 핵심 액션(로그인, 상태 변경 등)의 이력을 남기는 감사 로그입니다.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAuditLog {
    pub id: u64,                     // 데이터베이스 내부 식별용 PK
    pub user_id: u64,                // 액션을 수행한(혹은 대상이 된) 사용자의 PK
    pub action: UserAction,          // 수행된 액션의 카테고리
    pub description: Option<String>, // 액션에 대한 상세 부가 정보 (필요시 어떤 필드가 어떻게 바뀌었는지 문자열이나 JSON 기록)
    pub ip_address: Option<String>,  // 요청을 보낸 사용자의 접속 IP 주소
    pub user_agent: Option<String>,  // 접속 기기 및 브라우저 정보 (User-Agent)
    pub is_success: bool,            // 액션의 최종 성공 여부 (예: 로그인 실패 이력 관리용)
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: u64,
    pub code: String, // dot 으로 구분된 권한 코드 (예: "user.read", "user.write")
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: u64,
    pub name: String, // 역할 이름 (예: "admin", "user")
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub id: u64,
    pub role_id: u64,
    pub permission_id: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub id: u64,
    pub user_id: u64,
    pub role_id: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserWithRoles {
    #[sqlx(flatten)]
    pub user: User, // User의 모든 칼럼을 평탄화(Flatten)하여 자동 매핑
    pub role_ids: sqlx::types::Json<Vec<u64>>, // json_agg()로 조회된 role_id의 목록
}
