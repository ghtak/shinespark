pub enum Query {
    CreateUser,
    CreateIdentity,
    FindUser,
    UpdateUser,
}

impl shinespark::db::SqlStatement for Query {
    fn as_str(&self) -> &'static str {
        match self {
            Query::CreateUser => include_str!("../../sql/user_repository/create_user.sql"),
            Query::CreateIdentity => include_str!("../../sql/user_repository/create_identity.sql"),
            Query::FindUser => include_str!("../../sql/user_repository/find_user.sql"),
            Query::UpdateUser => include_str!("../../sql/user_repository/update_user.sql"),
        }
    }
}
