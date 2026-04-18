SELECT id, user_uid, token_hash, expires_at, created_at
FROM shs_iam_refresh_token
WHERE token_hash = $1
  AND expires_at > NOW()
