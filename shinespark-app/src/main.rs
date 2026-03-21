use shinespark::config::AppConfig;

extern crate shinespark;

fn main() {
    // .env -> .env.{RUN_MODE} -> env.local 의 순서로 loading
    AppConfig::load_dotenv();
    let config = AppConfig::new().expect("config load failed");
    shinespark::trace::init(&config.trace).expect("trace init failed");
    tracing::info!("Hello, world!");
}
