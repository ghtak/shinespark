use std::net::SocketAddr;

use axum::Router;

use crate::config::HttpConfig;

pub async fn run(router: Router, config: &HttpConfig) -> crate::Result<()> {
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        crate::Error::Internal(anyhow::anyhow!(e).context("failed to bind listener"))
    })?;
    tracing::info!("HTTP server running on {}", addr);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| crate::Error::Internal(anyhow::anyhow!(e).context("failed to serve")))?;
    Ok(())
}
