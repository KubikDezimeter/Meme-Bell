use axum::http::{header, HeaderValue, StatusCode};
use axum::routing::post;
use axum::{http::Response, response::{Html, IntoResponse}, routing::get, Router, Server, Json, body};
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::{BufReader, Read};
use std::{fs, thread};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use axum::body::Full;
use axum::extract::Path;

const RINGING_TIME: u64 = 15;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/ring", post(ring_post))
        .route("/api/get_ringtone_list", get(api_get_ringtone_list))
        .route("/api/get_ringtone/:ringtone", get(api_get_ringtone));
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
    let mp3 = File::open("ringtones/megalovania.mp3").unwrap();
    play_ringtone(mp3);

    StatusCode::OK
}

#[axum::debug_handler]
async fn api_get_ringtone_list() -> impl IntoResponse {
    let paths = fs::read_dir("ringtones").unwrap();
    let filenames: Vec<String> = paths.map(|path| path.unwrap().file_name().to_str().unwrap().to_string()).collect();
    Json(filenames)


    // Json(["megalovania.mp3", "Never Gonna Give You Up", "Nyan Cat", "500 Miles"])
}

async fn api_get_ringtone(Path(ringtone): Path<String>) -> impl IntoResponse {
    let path = ringtone.trim_start_matches('/').to_string();
    println!("Serving ringtones/{}", path);

    let cargo_manifest_dir: String = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let ringtone_path = PathBuf::from(cargo_manifest_dir).join("ringtones").join(path.clone());
    let ringtone_path: &std::path::Path = &ringtone_path.as_path();

    return match ringtone_path.exists() {
        true => {
            let mut file_contents = Vec::new();
            let mut file = File::open(&ringtone_path).expect("Unable to open audio file");
            file.read_to_end(&mut file_contents).expect("Unable to read audio file");
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("audio/mpeg").unwrap(),
                )
                .body(body::boxed(Full::from(file_contents)))
                .unwrap()
        },
        false => {
            println!("Requested resource ringtones/{} not found", path);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::boxed("Ringtone does not exist.".to_owned()))
                .unwrap()
        },
    }

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