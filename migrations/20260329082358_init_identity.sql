--------------------------------------------------------------------------------
-- 1. Create Enums (Idempotent)
--------------------------------------------------------------------------------

-- DO $$ BEGIN
--     CREATE TYPE auth_provider AS ENUM ('local', 'google', 'apple');
-- EXCEPTION
--     WHEN duplicate_object THEN null;
-- END $$;

-- DO $$ BEGIN
--     CREATE TYPE user_action AS ENUM ('login', 'logout', 'status_changed', 'credential_updated', 'profile_updated');
-- EXCEPTION
--     WHEN duplicate_object THEN null;
-- END $$;

--------------------------------------------------------------------------------
-- 2. Create Tables (Idempotent)
--------------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS  shs_iam_user (
    id BIGSERIAL PRIMARY KEY,
    uid UUID NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 삭제된 사용자를 제외한 이메일 중복 방지
CREATE UNIQUE INDEX shs_iam_user_email_active_idx
ON shs_iam_user (email)
WHERE status != 'deleted';

CREATE TABLE IF NOT EXISTS  shs_iam_user_identity (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    provider TEXT NOT NULL,
    provider_uid VARCHAR(255) NOT NULL,
    credential_hash VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    --FOREIGN KEY (user_id) REFERENCES  shs_iam_user (id) ON DELETE CASCADE,
    UNIQUE (user_id, provider, provider_uid)
);

CREATE TABLE IF NOT EXISTS  shs_iam_user_audit_log (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    action TEXT NOT NULL,
    description VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    is_success BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    --FOREIGN KEY (user_id) REFERENCES  shs_iam_user (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS  shs_iam_permission (
    id BIGSERIAL PRIMARY KEY,
    code VARCHAR(255) NOT NULL UNIQUE,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS  shs_iam_role (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS  shs_iam_role_permission (
    id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL,
    permission_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    --FOREIGN KEY (role_id) REFERENCES  shs_iam_role (id) ON DELETE CASCADE,
    --FOREIGN KEY (permission_id) REFERENCES  shs_iam_permission (id) ON DELETE CASCADE
    UNIQUE (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS  shs_iam_user_role (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    --FOREIGN KEY (user_id) REFERENCES  shs_iam_user (id) ON DELETE CASCADE,
    --FOREIGN KEY (role_id) REFERENCES  shs_iam_role (id) ON DELETE CASCADE
    UNIQUE (user_id, role_id)
);

--------------------------------------------------------------------------------
-- 3. Add Comments
--------------------------------------------------------------------------------

COMMENT ON TABLE  shs_iam_user IS '시스템의 핵심 식별 주체인 사용자 정보입니다.';
COMMENT ON COLUMN  shs_iam_user.id IS '데이터베이스 내부 식별용 PK (Auto Increment)';
COMMENT ON COLUMN  shs_iam_user.uid IS '외부 노출용 고유 식별자 (API 통신, 토큰 발급 등에 사용)';
COMMENT ON COLUMN  shs_iam_user.name IS '사용자 이름 또는 닉네임';
COMMENT ON COLUMN  shs_iam_user.email IS '연락 및 주요 인증 기준이 되는 이메일 주소';
COMMENT ON COLUMN  shs_iam_user.status IS '계정의 현재 활성화 상태';
COMMENT ON COLUMN  shs_iam_user.created_at IS '레코드 최초 생성 일시';
COMMENT ON COLUMN  shs_iam_user.updated_at IS '레코드 최종 수정 일시';

COMMENT ON TABLE  shs_iam_user_identity IS '사용자의 인증 수단 및 자격 증명(Credential) 정보입니다. (다중 플랫폼 로그인 지원)';
COMMENT ON COLUMN  shs_iam_user_identity.id IS '데이터베이스 내부 식별용 PK';
COMMENT ON COLUMN  shs_iam_user_identity.user_id IS '연관된 User의 PK (FK)';
COMMENT ON COLUMN  shs_iam_user_identity.provider IS '해당 인증의 제공자 (Local, Google, Apple 등)';
COMMENT ON COLUMN  shs_iam_user_identity.provider_uid IS '인증 제공자 측의 고유 식별자 (소셜 로그인의 경우 해당 플랫폼의 사용자 ID)';
COMMENT ON COLUMN  shs_iam_user_identity.credential_hash IS '(Local 인증 전용) 암호화된 비밀번호 해시값. 소셜 로그인 등 비밀번호가 없는 경우 NULL';
COMMENT ON COLUMN  shs_iam_user_identity.created_at IS '연동 정보 등록 일시';
COMMENT ON COLUMN  shs_iam_user_identity.updated_at IS '연동 정보 상태 변경 일시';

COMMENT ON TABLE  shs_iam_user_audit_log IS '사용자와 관련된 핵심 액션(로그인, 상태 변경 등)의 이력을 남기는 감사 로그입니다.';
COMMENT ON COLUMN  shs_iam_user_audit_log.id IS '데이터베이스 내부 식별용 PK';
COMMENT ON COLUMN  shs_iam_user_audit_log.user_id IS '액션을 수행한(혹은 대상이 된) 사용자의 PK';
COMMENT ON COLUMN  shs_iam_user_audit_log.action IS '수행된 액션의 카테고리';
COMMENT ON COLUMN  shs_iam_user_audit_log.description IS '액션에 대한 상세 부가 정보 (필요시 어떤 필드가 어떻게 바뀌었는지 문자열이나 JSON 기록)';
COMMENT ON COLUMN  shs_iam_user_audit_log.ip_address IS '요청을 보낸 사용자의 접속 IP 주소';
COMMENT ON COLUMN  shs_iam_user_audit_log.user_agent IS '접속 기기 및 브라우저 정보 (User-Agent)';
COMMENT ON COLUMN  shs_iam_user_audit_log.is_success IS '액션의 최종 성공 여부 (예: 로그인 실패 이력 관리용)';
COMMENT ON COLUMN  shs_iam_user_audit_log.created_at IS '이벤트가 발생한 일시';

COMMENT ON TABLE  shs_iam_permission IS '시스템의 권한 정보입니다.';
COMMENT ON COLUMN  shs_iam_permission.id IS '데이터베이스 내부 식별용 PK';
COMMENT ON COLUMN  shs_iam_permission.code IS 'dot 으로 구분된 권한 코드 (예: "user.read", "user.write")';
COMMENT ON COLUMN  shs_iam_permission.description IS '권한에 대한 상세 설명';
COMMENT ON COLUMN  shs_iam_permission.created_at IS '권한 데이터 최초 생성 일시';

COMMENT ON TABLE  shs_iam_role IS '시스템의 역할 정보입니다.';
COMMENT ON COLUMN  shs_iam_role.id IS '데이터베이스 내부 식별용 PK';
COMMENT ON COLUMN  shs_iam_role.name IS '역할 이름 (예: "admin", "user")';
COMMENT ON COLUMN  shs_iam_role.description IS '역할에 대한 상세 설명';
COMMENT ON COLUMN  shs_iam_role.created_at IS '역할 데이터 최초 생성 일시';

COMMENT ON TABLE  shs_iam_role_permission IS '역할에 부여된 권한 매핑 정보입니다.';
COMMENT ON COLUMN  shs_iam_role_permission.id IS '데이터베이스 내부 식별용 PK';
COMMENT ON COLUMN  shs_iam_role_permission.role_id IS '연관된 Role의 PK (FK)';
COMMENT ON COLUMN  shs_iam_role_permission.permission_id IS '연관된 Permission의 PK (FK)';
COMMENT ON COLUMN  shs_iam_role_permission.created_at IS '매핑 데이터 최초 생성 일시';

COMMENT ON TABLE  shs_iam_user_role IS '사용자에게 부여된 역할 매핑 정보입니다.';
COMMENT ON COLUMN  shs_iam_user_role.id IS '데이터베이스 내부 식별용 PK';
COMMENT ON COLUMN  shs_iam_user_role.user_id IS '연관된 User의 PK (FK)';
COMMENT ON COLUMN  shs_iam_user_role.role_id IS '연관된 Role의 PK (FK)';
COMMENT ON COLUMN  shs_iam_user_role.created_at IS '매핑 데이터 최초 생성 일시';


--------------------------------------------------------------------------------
-- 4. Add Default Data
--------------------------------------------------------------------------------

INSERT INTO shs_iam_role (name, description)
SELECT * FROM (
    SELECT 'admin' as name, '관리자'
    UNION ALL
    SELECT 'user' as name, '사용자'
) AS tmp
WHERE NOT EXISTS (
    SELECT 1 FROM shs_iam_role WHERE name = tmp.name
);


INSERT INTO shs_iam_permission (code, description)
SELECT * FROM (
    SELECT '*.*.all' as code, '모든 시스템 전체 권한' as description
    UNION ALL SELECT 'user.read.all' as code, '모든 사용자 조회' as description
    UNION ALL SELECT 'user.create.all' as code, '모든 사용자 생성' as description
    UNION ALL SELECT 'user.update.all' as code, '모든 사용자 수정' as description
    UNION ALL SELECT 'user.delete.all' as code, '모든 사용자 삭제' as description
    UNION ALL SELECT 'user.read.own' as code, '본인 정보 조회' as description
    UNION ALL SELECT 'user.update.own' as code, '본인 정보 수정' as description
    UNION ALL SELECT 'user.delete.own' as code, '본인 계정 탈퇴' as description
) AS tmp
WHERE NOT EXISTS (
    SELECT 1 FROM shs_iam_permission WHERE code = tmp.code
);

INSERT INTO shs_iam_role_permission (role_id, permission_id)
SELECT * FROM (
    SELECT sir.id as role_id, sip.id as permission_id
    FROM shs_iam_role sir, shs_iam_permission sip
    WHERE sir.name = 'admin'
        AND sip.code = '*.*.all'
    UNION ALL
    SELECT sir.id as role_id, sip.id as permission_id
    FROM shs_iam_role sir, shs_iam_permission sip
    WHERE sir.name = 'user'
        AND sip.code IN ('user.read.own', 'user.update.own', 'user.delete.own')
) AS tmp
WHERE NOT EXISTS (
    SELECT 1 FROM shs_iam_role_permission WHERE role_id = tmp.role_id AND permission_id = tmp.permission_id
);