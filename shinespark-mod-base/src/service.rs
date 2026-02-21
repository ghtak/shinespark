use crate::entity::SimpleEntity;

pub struct FindSimpleQuery {
    pub id: i64,
}

pub struct CreateSimpleCommand {
    pub name: String,
}

// 일반 Service
#[async_trait::async_trait]
pub trait SimpleService {
    async fn find_simple(
        &self,
        query: FindSimpleQuery,
    ) -> shinespark::Result<SimpleEntity>;
}

// Transaction을 사용하는 Service
// usecase 혹은 시나리오, feature 기반 호출에서 사용할 interface
#[async_trait::async_trait]
pub trait SimpleServiceTx {
    async fn create_simple(
        &self,
        h: &mut shinespark::db::AppDbHandle<'_>,
        command: CreateSimpleCommand,
    ) -> shinespark::Result<SimpleEntity>;
}

mod simple_service_impl;
pub use simple_service_impl::*;
