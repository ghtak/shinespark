INSERT INTO shs_iam_refresh_token (user_uid, token_hash, expires_at)
VALUES ($1, $2, $3)
ON CONFLICT (token_hash) DO NOTHING
