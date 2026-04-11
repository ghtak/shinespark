pub mod identity {
    use std::sync::Arc;

    use axum::{Router, extract::State};

    use crate::AppContainer;

    async fn login(State(_): State<Arc<AppContainer>>) -> &'static str {
        "hello"
    }

    pub fn routes() -> Router<Arc<AppContainer>> {
        Router::new().route("/identity/login", axum::routing::post(login))
    }
}
