SELECT rp.role_id, p.code
FROM shs_iam_role_permission rp
INNER JOIN shs_iam_permission p ON p.id = rp.permission_id
