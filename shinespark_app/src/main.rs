#[tokio::main]
async fn main() {
    let config = shinespark::config::AppConfig::new(
        shinespark::util::workspace_dir()
            .join("configs")
            .join("dev"),
    )
    .unwrap();
    println!("config: {:?}", config);
}
