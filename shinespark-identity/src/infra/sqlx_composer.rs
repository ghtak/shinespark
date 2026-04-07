use crate::entity::UserStatus;
use crate::service::{FindUserQuery, UpdateUserCommand};

impl shinespark::db::SqlComposer for FindUserQuery {
    fn compose<'q>(
        &'q self,
        query_builder: &mut sqlx::QueryBuilder<'q, shinespark::db::Driver>,
    ) -> shinespark::Result<()> {
        if let Some(id) = &self.id {
            query_builder.push(" AND u.id = ").push_bind(id);
        }
        if let Some(uid) = &self.uid {
            query_builder.push(" AND u.uid = ").push_bind(uid);
        }
        if let Some(email) = &self.email {
            query_builder.push(" AND u.email = ").push_bind(email);
        }
        if !self.with_deleted {
            query_builder
                .push(" AND u.status != ")
                .push_bind(UserStatus::Deleted.as_str());
        }
        query_builder.push(" GROUP BY u.id");
        Ok(())
    }
}

impl shinespark::db::SqlComposer for UpdateUserCommand {
    fn compose<'q>(
        &'q self,
        query_builder: &mut sqlx::QueryBuilder<'q, shinespark::db::Driver>,
    ) -> shinespark::Result<()> {
        if let Some(status) = &self.status {
            query_builder.push(", status = ").push_bind(status.as_str());
        }
        query_builder
            .push(" where id = ")
            .push_bind(&self.id)
            .push(" returning *");
        Ok(())
    }
}
