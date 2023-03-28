use axum::{
    routing::get,
    Router,
};
use std::io::Error;

pub async fn start_webserver(addr: String) -> Result<(), Error> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
            
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}