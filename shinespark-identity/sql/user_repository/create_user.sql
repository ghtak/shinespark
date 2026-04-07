INSERT INTO shs_iam_user (uid, name, email, status)
VALUES ($1, $2, $3, $4)
-- ON CONFLICT (email, status) DO UPDATE SET
--     name = EXCLUDED.name,
--     status = EXCLUDED.status
RETURNING *