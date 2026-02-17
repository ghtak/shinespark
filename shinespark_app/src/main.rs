#[tokio::main]
async fn main() {
    let config = shinespark::config::AppConfig::new(
        shinespark::util::workspace_dir().join("configs")
    )
    .unwrap();

    shinespark::logging::init_tracing(&config.logging)
        .expect("Failed to initialize logger");

    tracing::info!("app config: {:?}", config);
}
