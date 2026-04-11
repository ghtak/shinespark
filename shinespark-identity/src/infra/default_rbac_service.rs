use crate::services::RbacService;

pub struct DefaultRbacService {}

impl DefaultRbacService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl RbacService for DefaultRbacService {}
