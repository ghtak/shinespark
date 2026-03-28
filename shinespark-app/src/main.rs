use std::sync::Arc;

use axum::extract::State;
use shinespark::config::AppConfig;

extern crate shinespark;

#[derive(Clone)]
pub struct AppContainer {
    db: shinespark::db::Database,
}

impl AppContainer {
    pub fn new(db: shinespark::db::Database) -> Self {
        Self { db }
    }
}

#[tokio::main]
async fn main() {
    AppConfig::load_dotenv();
    let config = AppConfig::new().expect("failed to load config");
    shinespark::trace::init(&config.trace).expect("failed to init trace");
    let db = shinespark::db::Database::new(&config.database)
        .await
        .expect("failed to create database");
    let container = Arc::new(AppContainer::new(db));
    let router = axum::Router::new()
        .route(
            "/",
            axum::routing::get(|State(container): State<Arc<AppContainer>>| async move {
                let mut handle = container.db.handle();
                let result = sqlx::query("SELECT 1")
                    .execute(handle.inner())
                    .await
                    .unwrap();
                format!("Hello, world! {}", result.rows_affected())
            }),
        )
        .with_state(container);

    shinespark::http::run(router, &config.http)
        .await
        .expect("failed to run http server");
}
