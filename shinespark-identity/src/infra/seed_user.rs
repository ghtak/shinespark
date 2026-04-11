use std::sync::Arc;

use shinespark::db::{self};

use crate::{
    entities::UserStatus,
    usecases::{CreateUserCommand, FindUserQuery, InitialCredentials, UserUsecase},
};

pub async fn seed_admin(handle: &mut db::Handle<'_>, usecase: Arc<dyn UserUsecase>) {
    let user = usecase
        .find_user(
            handle,
            FindUserQuery::new().email("admin@shinespark.dev".into()),
        )
        .await
        .unwrap();
    if user.is_none() {
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
            .unwrap();
    }
}
