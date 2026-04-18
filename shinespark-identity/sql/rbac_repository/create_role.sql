INSERT INTO shs_iam_role (name, description)
VALUES ($1, $2)
RETURNING *
