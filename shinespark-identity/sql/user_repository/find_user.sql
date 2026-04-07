SELECT u.*
    ,(
        SELECT COALESCE(json_agg(role_id), '[]'::json)
        FROM shs_iam_user_role r
        WHERE r.user_id = u.id
    ) as role_ids
    ,(
        SELECT COALESCE(json_agg(i.*), '[]'::json)
        FROM shs_iam_user_identity i
        WHERE i.user_id = u.id
    ) as identities
FROM shs_iam_user u
WHERE 1 = 1
