pub struct CreateSimpleCommand {
    pub name: String,
}

// 업무, 시나리오 단위
#[async_trait::async_trait]
pub trait CreateSimpleUsecase {
    async fn execute(
        &self,
        command: CreateSimpleCommand,
    ) -> shinespark::Result<()>;
}

#[async_trait::async_trait]
pub trait DeleteAllSimpleUsecase {
    async fn execute(&self) -> shinespark::Result<()>;
}

mod simple_usecase_impl;
pub use simple_usecase_impl::*;
