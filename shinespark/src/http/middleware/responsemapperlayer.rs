pub async fn response_mapper_layer(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(request).await;
    match response.status() {
        s if s == axum::http::StatusCode::UNPROCESSABLE_ENTITY
            || s == axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE =>
        {
            let (mut parts, body) = response.into_parts();
            let body = axum::body::to_bytes(body, usize::MAX).await.unwrap();
            let new_body = serde_json::json!({
                "message": String::from_utf8_lossy(&body).to_string(),
            })
            .to_string();
            parts.headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );
            parts.headers.insert(
                axum::http::header::CONTENT_LENGTH,
                axum::http::HeaderValue::from(new_body.len()),
            );

            axum::response::Response::from_parts(
                parts,
                axum::body::Body::from(new_body),
            )
        }
        _ => response,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ax_body::Body;
    use axum::{
        Router, body as ax_body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_response_mapper_422() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    (StatusCode::UNPROCESSABLE_ENTITY, "invalid input")
                }),
            )
            .layer(axum::middleware::from_fn(response_mapper_layer));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body =
            ax_body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "invalid input");
    }

    #[tokio::test]
    async fn test_response_mapper_415() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    (StatusCode::UNSUPPORTED_MEDIA_TYPE, "bad format")
                }),
            )
            .layer(axum::middleware::from_fn(response_mapper_layer));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body =
            ax_body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["message"], "bad format");
    }

    #[tokio::test]
    async fn test_response_mapper_passthrough() {
        let app = Router::new()
            .route("/", get(|| async { (StatusCode::OK, "success") }))
            .layer(axum::middleware::from_fn(response_mapper_layer));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        // Default content type for string is text/plain
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );

        let body =
            ax_body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "success");
    }
}
