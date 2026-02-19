#[tokio::main]
async fn main() {
    let config = shinespark::config::AppConfig::new(
        shinespark::util::workspace_dir().join("configs"),
    )
    .expect("Failed to load app config");

    shinespark::logging::init_tracing(&config.logging)
        .expect("Failed to initialize logger");

    tracing::info!("app config: {:?}", config);

    let router = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }));

    shinespark::http::run(router, &config.server)
        .await
        .expect("Failed to run server");
}
