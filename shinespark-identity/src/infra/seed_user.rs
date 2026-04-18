use std::sync::Arc;

use shinespark::db;

use crate::{
    entities::UserStatus,
    usecases::{CreateUserCommand, FindUserQuery, InitialCredentials, RbacUsecase, UserUsecase},
};

pub async fn seed_admin(
    handle: &mut db::Handle<'_>,
    usecase: Arc<dyn UserUsecase>,
    rbac_usecase: Arc<dyn RbacUsecase>,
) {
    let existing = usecase
        .find_user(handle, FindUserQuery::new().email("admin@shinespark.dev".into()))
        .await
        .unwrap();

    let user_id = if let Some(u) = existing {
        u.user.id
    } else {
        usecase
            .create_user(
                handle,
                CreateUserCommand {
                    name: "admin".to_string(),
                    email: "admin@shinespark.dev".to_string(),
                    credentials: InitialCredentials::Local {
                        password: "password".to_string(),
                    },
                    status: UserStatus::Active,
                },
            )
            .await
            .unwrap()
            .user
            .id
    };

    rbac_usecase.assign_role_to_user(handle, user_id, "admin").await.unwrap();
}
