use axum::http::StatusCode;
use axum::routing::post;
use axum::{
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Router, Server,
};
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const RINGING_TIME: u64 = 15;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/ring", post(ring_post));
    let server = Server::bind(&"0.0.0.0:8989".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");

    server.await.unwrap();
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();

    Html(markup)
}

#[axum::debug_handler]
async fn indexmjs_get() -> impl IntoResponse {
    let javascript = tokio::fs::read_to_string("src/index.mjs").await.unwrap();

    Response::builder()
        .header("content-type", "application/javascript;charset=utf-8")
        .body(javascript)
        .unwrap()
}

#[axum::debug_handler]
async fn indexcss_get() -> impl IntoResponse {
    let css = tokio::fs::read_to_string("src/index.css").await.unwrap();

    Response::builder()
        .header("content-type", "text/css;charset=utf-8")
        .body(css)
        .unwrap()
}

async fn ring_post() -> impl IntoResponse {
    println!("Bell is ringing");
    let mp3 = File::open("sounds/megalovania.mp3").unwrap();
    play_ringtone(mp3);

    StatusCode::OK
}

/// Play the file 'sound' on the local machine
fn play_ringtone(ringtone: File) {
    thread::spawn(|| {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let bufreader = BufReader::new(ringtone);
        let source = Decoder::new(bufreader).unwrap();
        stream_handle.play_raw(source.convert_samples()).unwrap();
        sleep(Duration::from_secs(RINGING_TIME));
    });
}
