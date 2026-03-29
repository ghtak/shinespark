pub struct DefaultRbacService {}

impl DefaultRbacService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl super::RbacService for DefaultRbacService {}
