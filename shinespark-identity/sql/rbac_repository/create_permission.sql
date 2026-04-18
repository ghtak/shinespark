INSERT INTO shs_iam_permission (code, description)
VALUES ($1, $2)
RETURNING *
