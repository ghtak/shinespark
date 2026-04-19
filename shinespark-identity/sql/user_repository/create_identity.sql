INSERT INTO
    shs_iam_user_identity (user_id, provider, provider_uid, credential_hash)
VALUES ($1, $2, $3, $4)
ON CONFLICT (user_id, provider, provider_uid) DO UPDATE SET
    credential_hash = COALESCE(EXCLUDED.credential_hash, shs_iam_user_identity.credential_hash),
    updated_at = NOW()
RETURNING *