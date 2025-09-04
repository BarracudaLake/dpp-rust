//
//     DDP - main exec
//     Programmed by Barracudalake
//     Newest,    Set
//     2.0.25
//
const VERSION: &str = env!("CARGO_PKG_VERSION");
const ADDRESS: &str = "127.0.0.1:8487"; // use constants for now, remake as CLI later TODO

// these versions are hard-coded, theres no need to currently make them dynamic
const SUPPORTED_VERSIONS: Vec<&str> = vec![VERSION];

use std::fs;
use std::fs::OpenOptions;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Request, State},
    body::Body,
    routing::{get, post, options},
    response::{IntoResponse},
    http::{StatusCode, HeaderValue, header, Response},
    Json, Router
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::{TraceLayer}
};
use tracing::{info, Span};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt
};
use tokio::{
    sync::Mutex as TokioMutex,
};

// CONFIG START
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Configuration {
    server: ServerConfiguration
}
#[derive(Deserialize, Serialize, Debug, Clone)]
struct ServerConfiguration {
    address: String,
    featureflags: u8,
}

// CONFIG END

#[derive(Deserialize, Serialize)]
struct INITVersionDataRequest {
    version: String,
    supported_versions: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct INITVersionDataResponse {
    version: String,
    supported_versions: Vec<String>,
    used: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct ServerList {
    server: Vec<ServerListEntry>
}
#[derive(Deserialize, Serialize, Clone)]
struct ServerListEntry {
    version: String,
    ip: String,
    hash: String,
    rating: u16,
    ff: u8 //feature flags
    //necessity: ?
}

#[derive(Clone)]
struct AppState {
    configuration: Arc<Configuration>, // should be ROnly but who knows, it might change in the future.
    server_list: Arc<TokioMutex<ServerList>>,
}

#[tokio::main]
async fn main() {
    println!("DDP Starting..");

    let configuration = check_config().unwrap_or_else(|err| {
        panic!("DDP Error [1100]: no config: {}", err);
    });

    let server_list = check_slist().unwrap_or_else(|err| {
        panic!("DDP Error [1101]: no server list: {}", err);
    });

    let state = AppState {
        configuration: Arc::new(configuration),
        server_list: Arc::new(TokioMutex::new(server_list)),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .init(); // init tracer

    //Routing Start
    let app = Router::new()

        .route("/", get( root ))
        .route("/i/init", options( init ))

        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any)
        )
        .layer(
            TraceLayer::new_for_http().on_response(|res: &Response<Body>, latency:Duration, span:&Span| {
                let status = res.status();
                let _cspan = span.enter();
                info!("[{}] {:?}/DPP/1.0 - with {:?}", status.as_u16(), res.version(), latency);
            })
        )

        .with_state(state)

        ;
    //Routing End

    info!("DDP Router has been built!");

    let listener = tokio::net::TcpListener::bind(ADDRESS).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    info!("DDP is now listening..");
}

async fn root() -> impl IntoResponse {
    // handles "/" requests
    // IDEA: make this a web interface for dpp?

    (StatusCode::BAD_REQUEST, "This route does not exist as it is considered invalid on DPP");
}

async fn init(State(state): AppState, body: String) -> impl IntoResponse {
    let data = match toml::from_str(&*body) : INITVersionDataRequest {
        Ok(configuration) => configuration,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "This route does not exist as it is considered invalid on DPP");
        }
    };

    // Lets compare versions!


}



/*
 * It is true that often a file has to be returned or downloaded. Here's functions that does that
 */
/*async fn return_file(path: impl AsRef<Path>) -> impl IntoResponse {
    let file = match File::open(&path).await {
        Ok(file) => file,
        Err(_) => return Err((StatusCode::NOT_FOUND, "what the fuck are you talking about".to_string()))
    };

    let content_type = match from_path(&path).first_raw() {
        Some(mime) => mime,
        None => return Err((StatusCode::BAD_REQUEST, "whatever the FUCK you did with the file fucked it up so much\n how did you even".to_string()))
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
        (
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("inline"),
        ),
    ];

    Ok((headers, body))
}
async fn download_file(path: impl AsRef<Path>) -> impl IntoResponse {
    let file = match File::open(&path).await {
        Ok(file) => file,
        Err(_) => return Err((StatusCode::NOT_FOUND, "what the fuck are you talking about".to_string()))
    };

    let content_type = match from_path(&path).first_raw() {
        Some(mime) => mime,
        None => return Err((StatusCode::BAD_REQUEST, "whatever the FUCK you did with the file fucked it up so much\n how did you even".to_string()))
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let filename = format!(
        "attachment; filename=\"{:?}\"",
        path.as_ref()
            .file_name()
            .unwrap()
    );

    let cd_wrap = HeaderValue::from_str(&filename).unwrap();

    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
        (
            header::CONTENT_DISPOSITION,
            cd_wrap,
        ),
    ];

    Ok((headers, body))
}*/

fn check_config() -> Result<Configuration, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("/etc/dpp/config.toml")?;

    Ok(toml::from_str(&content)?)
}

fn check_slist() -> Result<ServerList, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("/etc/dpp/server_list.toml")?;

    Ok(toml::from_str(&content)?)
}