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
    .map_err(|e| anyhow::Error::new(e).context("tcp bind failed"))?;
    axum::serve(listener, router)
        .await
        .map_err(|e| anyhow::Error::new(e).context("http serve failed"))?;
    Ok(())
}
