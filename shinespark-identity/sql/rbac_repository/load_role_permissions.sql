SELECT
    rp.role_id,
    p.code
FROM
    shs_iam_role_permission rp
INNER JOIN
    shs_iam_permission p ON 1=1
    AND p.id = rp.permission_id
