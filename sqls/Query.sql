SELECT *
FROM shs_iam_user siu
WHERE siu.status != 'deleted';


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
FROM
    shs_iam_user u,
    shs_iam_user_identity i
WHERE 1 = 1
    AND u.id = i.user_id
    AND u.status <> 'deleted'
    AND i.provider = 'local'
--    AND i.provider_uid = $2
;

select * from public.shs_iam_user_role;