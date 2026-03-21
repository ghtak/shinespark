use shinespark::config::AppConfig;

extern crate shinespark;

fn main() {
    let config = AppConfig::new().expect("config load failed");
    shinespark::trace::init(&config.trace).expect("trace init failed");
    tracing::info!("Hello, world!");
}
