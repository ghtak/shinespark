use crate::service::FindUserQuery;

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
