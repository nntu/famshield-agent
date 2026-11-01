use crate::ui::LOCAL_DASHBOARD_HTML;
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../../web/dist/"]
struct WebAssets;

// ================= Handlers =================

pub(crate) async fn handle_index() -> impl IntoResponse {
    if let Some(file) = WebAssets::get("index.html") {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(file.data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    Html(LOCAL_DASHBOARD_HTML).into_response()
}

pub(crate) async fn handle_assets(Path(asset_path): Path<String>) -> impl IntoResponse {
    let clean_path = asset_path.trim_start_matches('/').replace("..", "");
    let full_path = format!("assets/{}", clean_path);

    if let Some(file) = WebAssets::get(&full_path) {
        let mime = mime_guess::from_path(&clean_path).first_or_octet_stream();

        return Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(axum::body::Body::from(file.data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    StatusCode::NOT_FOUND.into_response()
}

pub(crate) async fn handle_fallback(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Do not intercept missing API routes
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Check if the requested file exists directly in WebAssets (e.g. favicon.ico, vite.svg)
    if let Some(file) = WebAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(axum::body::Body::from(file.data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    // Fallback to index.html for SPA client-side routing
    handle_index().await.into_response()
}
