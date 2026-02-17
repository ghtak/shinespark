-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    uid UUID UNIQUE NOT NULL DEFAULT GEN_RANDOM_UUID(),
    nickname VARCHAR(255),
    email VARCHAR(255) UNIQUE NOT NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'USER',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE users IS '사용자 핵심 정보 구성';
COMMENT ON COLUMN users.uid IS '외부 노출용 UUID (보안 및 API용)';
COMMENT ON COLUMN users.nickname IS '사용자 닉네임 (선택 사항)';
COMMENT ON COLUMN users.email IS '사용자 식별 및 알림용 이메일 (Unique)';
COMMENT ON COLUMN users.role IS '사용자 권한 (e.g. USER, ADMIN)';

CREATE TABLE IF NOT EXISTS user_identities (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL, -- Logical link to users(id)
    provider VARCHAR(32) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    credential_hash VARCHAR(255), -- local 인증용 (OAuth2는 NULL)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_user_id)
);

-- 물리적 FK는 제거하되, 검색 성능을 위해 user_id 인덱스는 추가합니다.
CREATE INDEX idx_user_identities_user_id ON user_identities(user_id);

COMMENT ON TABLE user_identities IS '외부 OAuth2 인증 정보 및 로컬 인증 정보 통합 관리';
COMMENT ON COLUMN user_identities.user_id IS 'users 테이블의 id와 논리적 연결 (FK 제약 없음)';
COMMENT ON COLUMN user_identities.provider IS '인증 공급자 (e.g. google, apple, local)';
COMMENT ON COLUMN user_identities.provider_user_id IS '인증 공급자에서 제공하는 사용자 고유 ID (local일 경우 email 등)';
COMMENT ON COLUMN user_identities.credential_hash IS '로컬 로그인용 비밀번호 해시 (provider가 local인 경우 사용)';

