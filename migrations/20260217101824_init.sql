-- Add migration script here

CREATE TABLE IF NOT EXISTS ss_id_users (
    id BIGSERIAL PRIMARY KEY,
    uid UUID UNIQUE NOT NULL DEFAULT GEN_RANDOM_UUID(),
    name VARCHAR(255),
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE ss_id_users IS '사용자 핵심 정보 구성';
COMMENT ON COLUMN ss_id_users.uid IS '외부 노출용 UUID (보안 및 API용)';
COMMENT ON COLUMN ss_id_users.name IS '사용자 이름 (선택 사항)';
COMMENT ON COLUMN ss_id_users.email IS '사용자 식별 및 알림용 이메일 (Unique)';

CREATE TABLE IF NOT EXISTS ss_id_user_identities (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL, -- Logical link to users(id)
    provider VARCHAR(32) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    credential_hash VARCHAR(255), -- local 인증용 (OAuth2는 NULL)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_user_id)
);

-- 물리적 FK는 제거하되, 검색 성능을 위해 user_id 인덱스는 추가합니다.
CREATE INDEX idx_ss_id_user_identities_user_id ON ss_id_user_identities(user_id);

COMMENT ON TABLE ss_id_user_identities IS '외부 OAuth2 인증 정보 및 로컬 인증 정보 통합 관리';
COMMENT ON COLUMN ss_id_user_identities.user_id IS 'ss_id_users 테이블의 id와 논리적 연결 (FK 제약 없음)';
COMMENT ON COLUMN ss_id_user_identities.provider IS '인증 공급자 (e.g. google, apple, local)';
COMMENT ON COLUMN ss_id_user_identities.provider_user_id IS '인증 공급자에서 제공하는 사용자 고유 ID (local일 경우 email 등)';
COMMENT ON COLUMN ss_id_user_identities.credential_hash IS '로컬 로그인용 비밀번호 해시 (provider가 local인 경우 사용)';

-- RBAC (Role-Based Access Control)
CREATE TABLE IF NOT EXISTS ss_id_roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ss_id_permissions (
    id SERIAL PRIMARY KEY,
    code VARCHAR(128) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ss_id_user_roles (
    user_id BIGINT NOT NULL,
    role_id INT NOT NULL,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS ss_id_role_permissions (
    role_id INT NOT NULL,
    permission_id INT NOT NULL,
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX idx_ss_id_user_roles_user_id ON ss_id_user_roles(user_id);
CREATE INDEX idx_ss_id_role_permissions_role_id ON ss_id_role_permissions(role_id);

COMMENT ON TABLE ss_id_roles IS '사용자 역할 정의 (ADMIN, USER 등)';
COMMENT ON TABLE ss_id_permissions IS '세부 권한 코드 정의 (user:read, post:write 등)';
COMMENT ON TABLE ss_id_user_roles IS '사용자-역할 매핑';
COMMENT ON TABLE ss_id_role_permissions IS '역할-권한 매핑';
