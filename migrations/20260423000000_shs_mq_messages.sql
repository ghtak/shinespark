CREATE TABLE IF NOT EXISTS shs_mq_messages (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    topic        TEXT        NOT NULL,
    payload      JSONB       NOT NULL,
    status       TEXT        NOT NULL DEFAULT 'pending',
    attempts     INT         NOT NULL DEFAULT 0,
    max_attempts INT         NOT NULL DEFAULT 3,
    locked_at    TIMESTAMPTZ,
    done_at      TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shs_mq_pending
    ON shs_mq_messages (topic, created_at)
    WHERE status = 'pending';
