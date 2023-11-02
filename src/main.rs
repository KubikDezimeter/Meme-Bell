use axum::{routing::get, Router, Server, response::{Html, IntoResponse}};

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(root_get));
    let server = Server::bind(&"0.0.0.0:8989".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");

    server.await.unwrap();
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    Html(include_str!("index.html"))
}
