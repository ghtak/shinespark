use crate::config::ServerConfig;

pub async fn run(
    router: axum::Router,
    config: &ServerConfig,
) -> crate::Result<()> {
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.host.as_str(),
        config.port
    ))
    .await
    .map_err(|e| anyhow::anyhow!("tcp bind failed: {:?}", e))?;
    axum::serve(listener, router)
        .await
        .map_err(|e| anyhow::anyhow!("http serve failed: {:?}", e))?;
    Ok(())
}
