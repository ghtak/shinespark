pub enum Query {
    CreateUser,
    CreateIdentity,
    FindUser,
    DeleteUser,
}

impl shinespark::db::SqlStatement for Query {
    fn as_str(&self) -> &'static str {
        match self {
            Query::CreateUser => include_str!("../../sql/user_repository/create_user.sql"),
            Query::CreateIdentity => include_str!("../../sql/user_repository/create_identity.sql"),
            Query::FindUser => include_str!("../../sql/user_repository/find_user.sql"),
            Query::DeleteUser => include_str!("../../sql/user_repository/delete_user.sql"),
        }
    }
}

// impl<'q, O> From<Query>
//     for QueryAs<
//         'q,
//         shinespark::db::Driver,
//         O,
//         <shinespark::db::Driver as sqlx::Database>::Arguments<'q>,
//     >
// where
//     O: for<'r> sqlx::FromRow<'r, <shinespark::db::Driver as sqlx::Database>::Row>,
// {
//     fn from(query: Query) -> Self {
//         sqlx::query_as(query.as_str())
//     }
// }
