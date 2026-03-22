use shinespark::config::AppConfig;

extern crate shinespark;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    AppConfig::load_dotenv();
    let config = AppConfig::new().expect("failed to load config");
    shinespark::trace::init(&config.trace).expect("failed to init trace");
    shinespark::http::run(
        axum::Router::new().route("/", axum::routing::get(|| async { "Hello, world!" })),
        &config.http,
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
