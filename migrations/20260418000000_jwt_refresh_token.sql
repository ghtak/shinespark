CREATE TABLE shs_iam_refresh_token (
    id          BIGSERIAL PRIMARY KEY,
    user_uid    UUID NOT NULL,
    token_hash  VARCHAR(255) NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_token_user_uid ON shs_iam_refresh_token(user_uid);
