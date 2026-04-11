use crate::usecases::RbacUsecase;

pub struct DefaultRbacUsecase {}

impl DefaultRbacUsecase {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RbacUsecase for DefaultRbacUsecase {}
