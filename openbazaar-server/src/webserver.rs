use async_trait::async_trait;
use axum::extract::FromRequest;
use axum::http::request::Parts;
use axum::http::Request as HttpRequest;
use axum::http::Request;
use axum::{
    extract::{Extension, FromRequestParts, Path},
    http::{Response, StatusCode},
    response::Html,
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::io::Error;
use std::sync::Arc;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};

struct AppUser {
    username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AppUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        println!("req {:?}", parts.headers.get("Referer"));
        Ok(AppUser {
            username: "test".to_string(),
        })
    }
}

pub async fn start_webserver(addr: String) -> Result<(), Error> {
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/auth", get(root));

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn root(user: AppUser) -> Result<String, Infallible> {
    Ok(format!("Hello my man, {}!", user.username))
}
