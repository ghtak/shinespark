use crate::service::{FindUserQuery, UpdateUserCommand};

impl shinespark::db::QueryFilter for FindUserQuery {
    fn apply<'q>(
        &'q self,
        query_builder: &mut sqlx::QueryBuilder<'q, shinespark::db::Driver>,
    ) -> shinespark::Result<()> {
        shinespark::db::bind_opt(query_builder, " AND u.id = ", &self.id);
        shinespark::db::bind_opt(query_builder, " AND u.uid = ", &self.uid);
        shinespark::db::bind_opt(query_builder, " AND u.email = ", &self.email);
        Ok(())
    }
}

impl shinespark::db::QueryFilter for UpdateUserCommand {
    fn apply<'q>(
        &'q self,
        query_builder: &mut sqlx::QueryBuilder<'q, shinespark::db::Driver>,
    ) -> shinespark::Result<()> {
        if let Some(status) = &self.status {
            query_builder.push(", status = ");
            query_builder.push_bind(status.as_str());
        }
        query_builder.push(" where id = ");
        query_builder.push_bind(&self.id);
        query_builder.push(" returning *");
        Ok(())
    }
}
