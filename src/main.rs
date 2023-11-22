use axum::body::Full;
use axum::extract::{DefaultBodyLimit, Multipart, Path};
use axum::http::{header, HeaderValue, StatusCode};
use axum::routing::{post, put};
use axum::{
    body,
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router, Server,
};
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{fs, thread};
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize)]
struct Settings {
    ringing_time: usize,
}

#[tokio::main]
async fn main() {
    // Initialize folder structure
    fs::create_dir_all(format!("{}/ringtones/.trash", get_root_path()))
        .expect("Couldn't create directories");

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.mjs", get(indexmjs_get))
        .route("/index.css", get(indexcss_get))
        .route("/api/ring", post(ring_post))
        .route("/api/ringtone_list", get(api_ringtone_list_get))
        .route("/api/ringtone/:ringtone", get(api_ringtone_get))
        .route("/api/upload", post(api_upload_post))
        .route("/api/remove/:ringtone", post(api_remove_post))
        .route("/api/settings/ringing_time", get(api_setting_ringing_time_get))
        .route("/api/settings/ringing_time", put(api_setting_ringing_time_put))
        .layer(DefaultBodyLimit::max(2_usize.pow(30)));
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
    let ringtone_list = get_ringtone_list();
    let rand_index = fastrand::usize(..ringtone_list.len());
    let ringtone = &ringtone_list[rand_index];
    let mp3 = File::open(format!("ringtones/{ringtone}")).unwrap();
    println!("Playing 'ringtones/{ringtone}'");
    play_ringtone(mp3);

    StatusCode::OK
}

#[axum::debug_handler]
async fn api_ringtone_list_get() -> impl IntoResponse {
    Json::into_response(Json(get_ringtone_list()))
}

#[axum::debug_handler]
async fn api_ringtone_get(Path(ringtone): Path<String>) -> impl IntoResponse {
    let path = ringtone.trim_start_matches('/').to_string();
    println!("Serving ringtones/{}", path);

    let cargo_manifest_dir: String = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let ringtone_path = PathBuf::from(cargo_manifest_dir)
        .join("ringtones")
        .join(path.clone());
    let ringtone_path: &std::path::Path = &ringtone_path.as_path();

    return match ringtone_path.exists() {
        true => {
            let mut file_contents = Vec::new();
            let mut file = File::open(&ringtone_path).expect("Unable to open audio file");
            file.read_to_end(&mut file_contents)
                .expect("Unable to read audio file");
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("audio/mpeg").unwrap(),
                )
                .body(body::boxed(Full::from(file_contents)))
                .unwrap()
        }
        false => {
            println!("Requested resource ringtones/{} not found", path);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::boxed("Ringtone does not exist.".to_owned()))
                .unwrap()
        }
    };
}

#[axum::debug_handler]
async fn api_upload_post(mut multipart: Multipart) -> impl IntoResponse {
    let cargo_manifest_dir = get_root_path();
    // let mut filename: Option<String> = Option::None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();
        if field_name == "file" {
            let filename = field.file_name().unwrap().to_string();
            let data = field.bytes().await.unwrap();
            println!("File '{}' received with size of {}", filename, data.len());
            let mut file: File = File::create(
                PathBuf::from(cargo_manifest_dir.clone())
                    .join("ringtones")
                    .join(filename.clone()),
            )
            .unwrap();
            match file.write_all(&data) {
                Ok(_) => {
                    println!("File uploaded successfully");
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(body::boxed("Failed to save video to disk.".to_owned()))
                        .unwrap();
                }
            };
            let _ = file.flush();
            let _ = file.sync_all();
            break;
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(body::boxed(format!("Video uploaded successfully")))
        .unwrap()
}

#[axum::debug_handler]
async fn api_remove_post(Path(ringtone): Path<String>) -> impl IntoResponse {
    let cargo_manifest_dir = get_root_path();
    let filename = ringtone.trim_start_matches('/').to_string();
    println!("Moving {filename} to trash");
    match fs::rename(
        format!("{cargo_manifest_dir}/ringtones/{filename}"),
        format!("{cargo_manifest_dir}/ringtones/.trash/{filename}"),
    ) {
        Ok(()) => Response::builder()
            .status(StatusCode::OK)
            .body(body::boxed("".to_owned()))
            .unwrap(),
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::boxed(
                format!("Problem moving the file to trash: {err}").to_owned(),
            ))
            .unwrap(),
    }
}

#[axum::debug_handler]
async fn api_setting_ringing_time_get() -> impl IntoResponse {
    return Response::builder()
        .status(StatusCode::OK)
        .body(body::boxed(get_ringing_time().to_string()))
        .unwrap()
}

#[axum::debug_handler]
async fn api_setting_ringing_time_put(body: String) -> impl IntoResponse {
    let ringing_time_result = body.trim().parse::<usize>();
    match ringing_time_result {
        Ok(ringing_time) => {
            set_ringing_time(ringing_time);
            Response::builder()
                .status(StatusCode::OK)
                .body(body::boxed("".to_owned()))
                .unwrap()
        }
        Err(error) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(body::boxed(format!("Couldn't parse request body as number: {error}").to_owned()))
            .unwrap()
    }
}

/// Play the file 'sound' on the local machine
fn play_ringtone(ringtone: File) {
    thread::spawn(|| {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let bufreader = BufReader::new(ringtone);
        let source = Decoder::new(bufreader).unwrap();
        stream_handle.play_raw(source.convert_samples()).unwrap();
        sleep(Duration::from_secs(get_ringing_time() as u64));
    });
}

fn get_ringtone_list() -> Vec<String> {
    let entries = fs::read_dir(format!("{}/ringtones", get_root_path())).expect("Encountered an error while reading the 'ringtones' folder");
    let filenames: Vec<String> = entries
        .map(|entry| entry.unwrap())
        .filter(|entry| !entry.path().is_dir())
        .map(|entry| entry.file_name().to_str().unwrap().to_string())
        .collect();
    filenames
}

fn get_root_path() -> String {
    std::env::var("CARGO_MANIFEST_DIR").unwrap()
}

fn get_ringing_time() -> usize {
    let file = match File::open("settings.json") {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                set_ringing_time(5);
                File::open("settings.json").expect("Failed to initialize settings.json")
            },
            other_error => panic!("Couldn't create a settings file: {:?}", other_error),
        },
    };
    let bufreader = BufReader::new(file);
    let setting: Settings = serde_json::from_reader(bufreader).expect("Couldn't read settings from file");
    setting.ringing_time
}

fn set_ringing_time(ringing_time: usize) {
    let settings = Settings { ringing_time };
    let file = File::create("settings.json").expect("Unable to create settings file");
    let bufwriter = BufWriter::new(file);
    serde_json::to_writer_pretty(bufwriter, &settings).expect("Failed writing");
}