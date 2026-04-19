use shinespark::db::{SqlBuilderExt, SqlStatement};

use crate::entities::{AuthProvider, User, UserAggregate, UserIdentity, UserStatus};
use crate::repositories::UserRepository;
use crate::usecases::{FindUserQuery, UpdateUserCommand};

enum UserQuery {
    CreateIdentity,
    FindUser,
    FindUserByIdentity,
}

impl SqlStatement for UserQuery {
    fn as_str(&self) -> &'static str {
        match self {
            UserQuery::CreateIdentity => {
                include_str!("../../sql/user_repository/create_identity.sql")
            }
            UserQuery::FindUser => include_str!("../../sql/user_repository/find_user.sql"),
            UserQuery::FindUserByIdentity => {
                include_str!("../../sql/user_repository/find_user_by_identity.sql")
            }
        }
    }
}

pub struct SqlxUserRepository {}

impl SqlxUserRepository {
    pub fn new() -> Self {
        Self {}
    }
}

mod rows {
    use crate::entities::{User, UserAggregate, UserIdentity};

    #[derive(sqlx::FromRow)]
    pub struct UserAggregateRow {
        #[sqlx(flatten)]
        pub user: User,
        pub role_ids: sqlx::types::Json<Vec<i64>>,
        pub identities: sqlx::types::Json<Vec<UserIdentity>>,
    }

    impl From<UserAggregateRow> for UserAggregate {
        fn from(row: UserAggregateRow) -> Self {
            Self {
                user: row.user,
                role_ids: row.role_ids.0,
                identities: row.identities.0,
            }
        }
    }
}

#[async_trait::async_trait]
impl UserRepository for SqlxUserRepository {
    async fn create_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user: User,
    ) -> shinespark::Result<User> {
        let user = "INSERT INTO shs_iam_user (uid, name, email, status) VALUES ($1, $2, $3, $4) RETURNING *"
            .as_query_as::<User>()
            .bind(&user.uid)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.status.as_str())
            .fetch_one(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(user)
    }

    async fn create_identity(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        user_identity: UserIdentity,
    ) -> shinespark::Result<UserIdentity> {
        let user_identity = UserQuery::CreateIdentity
            .as_query_as::<UserIdentity>()
            .bind(&user_identity.user_id)
            .bind(&user_identity.provider.as_str())
            .bind(&user_identity.provider_uid)
            .bind(&user_identity.credential_hash)
            .fetch_one(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(user_identity)
    }

    async fn find_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        query: FindUserQuery,
    ) -> shinespark::Result<Option<UserAggregate>> {
        let mut b = UserQuery::FindUser.as_builder();
        b.push_option(" AND u.id = ", &query.id);
        b.push_option(" AND u.uid = ", &query.uid);
        b.push_option(" AND u.email = ", &query.email);
        if !query.with_deleted {
            b.push(" AND u.status != ").push_bind(UserStatus::Deleted.as_str());
        }
        let row = b
            .build_query_as::<rows::UserAggregateRow>()
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(row.map(rows::UserAggregateRow::into))
    }

    async fn update_user(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        command: UpdateUserCommand,
    ) -> shinespark::Result<User> {
        let status = command.status.as_ref().map(|s| s.as_str());
        let user = "UPDATE shs_iam_user SET updated_at = NOW()"
            .as_builder()
            .push_option(", status = ", &status)
            .push(" where id = ")
            .push_bind(&command.id)
            .push(" returning *")
            .build_query_as::<User>()
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        if user.is_none() {
            return Err(shinespark::Error::NotFound);
        }
        Ok(user.unwrap())
    }

    async fn find_user_by_identity(
        &self,
        handle: &mut shinespark::db::Handle<'_>,
        provider: AuthProvider,
        provider_uid: String,
    ) -> shinespark::Result<Option<UserAggregate>> {
        let row = UserQuery::FindUserByIdentity
            .as_query_as::<rows::UserAggregateRow>()
            .bind(provider.as_str())
            .bind(provider_uid)
            .fetch_optional(handle.inner())
            .await
            .map_err(|e| shinespark::Error::DatabaseError(anyhow::anyhow!(e)))?;
        Ok(row.map(|r| UserAggregate {
            user: r.user,
            role_ids: r.role_ids.0,
            identities: r.identities.0,
        }))
    }
}
