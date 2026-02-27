use crate::entity::User;
use shinespark::Result;

pub struct RegisterLocalUserReq {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[async_trait::async_trait]
pub trait RegisterLocalUserUseCase: Sync + Send {
    async fn execute(&self, req: RegisterLocalUserReq) -> Result<User>;
}
