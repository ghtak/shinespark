-- Add migration script here

INSERT INTO ss_id_users (uid, name, email, status)
SELECT gen_random_uuid(), 'admin', 'admin@shinespark.com', 'active'
WHERE NOT EXISTS (
    SELECT 1 FROM ss_id_users WHERE email = 'admin@shinespark.com'
);

INSERT INTO ss_id_user_identities (user_id, provider, provider_user_id, credential_hash)
SELECT
    (SELECT id FROM ss_id_users WHERE email = 'admin@shinespark.com'),
    'local',
    'admin@shinespark.com',
    '$argon2id$v=19$m=8,t=1,p=1$kPQvqYttwpgWoqNnmr7Oaw$xFLCzel1UIBGZzm9guwzSHlh11WaaE4XkKIHRceZ2zY'
    -- admin
WHERE NOT EXISTS (
    SELECT 1 FROM ss_id_user_identities WHERE provider_user_id = 'admin@shinespark.com'
);

INSERT INTO ss_id_roles (name)
SELECT 'admin'
WHERE NOT EXISTS (
    SELECT 1 FROM ss_id_roles WHERE name = 'admin'
);

INSERT INTO ss_id_permissions (code, description)
SELECT 'admin:all', 'Admin all permission'
WHERE NOT EXISTS (
    SELECT 1 FROM ss_id_permissions WHERE code = 'admin:all'
);

INSERT INTO ss_id_role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM ss_id_roles WHERE name = 'admin'),
    (SELECT id FROM ss_id_permissions WHERE code = 'admin:all')
WHERE NOT EXISTS (
    SELECT 1 FROM ss_id_role_permissions
    WHERE
        role_id = (SELECT id FROM ss_id_roles WHERE name = 'admin') AND
        permission_id = (SELECT id FROM ss_id_permissions WHERE code = 'admin:all')
    );

INSERT INTO ss_id_user_roles (user_id, role_id)
SELECT
    (SELECT id FROM ss_id_users WHERE email = 'admin@shinespark.com'),
    (SELECT id FROM ss_id_roles WHERE name = 'admin')
WHERE NOT EXISTS (
    SELECT 1 FROM ss_id_user_roles WHERE user_id = (SELECT id FROM ss_id_users WHERE email = 'admin@shinespark.com')
);
