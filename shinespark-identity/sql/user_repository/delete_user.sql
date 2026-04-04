UPDATE shs_iam_user
SET status = $1, updated_at = now()
WHERE id = $2
RETURNING *