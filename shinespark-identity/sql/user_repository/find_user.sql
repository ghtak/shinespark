SELECT u.*,
    COALESCE(
        json_agg(r.role_id) FILTER (
            WHERE r.role_id IS NOT NULL
        ),
        '[]'::json
    ) as role_ids
FROM shs_iam_user u
    LEFT JOIN shs_iam_user_role r ON u.id = r.user_id
WHERE 1 = 1

-- GROUP BY u.id