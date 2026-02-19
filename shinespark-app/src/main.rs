#[tokio::main]
async fn main() {
    let config = shinespark::config::AppConfig::new(
        shinespark::util::workspace_dir().join("configs"),
    )
    .expect("Failed to load app config");

    shinespark::trace::init(&config.trace).expect("Failed to initialize trace");

    tracing::info!("app config: {:?}", config);

    let router = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .layer(axum::middleware::from_fn(
            shinespark::http::middleware::trace_layer,
        ));

    shinespark::http::run(router, &config.server)
        .await
        .expect("Failed to run server");
}
