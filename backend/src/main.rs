use axum::body::{boxed, Body};
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::routing::post;
use axum::{routing::get, Router};
use clap::Parser;
use handler::api::{check_room_exist_handler, check_user_exist_handler};
use handler::websocket::websocket_handler;
use lib::{IP_ADDRESS, PORT};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::log;

use crate::server::ServerApp;

mod handler;
mod server;
mod syncing_system;

#[derive(Parser, Debug, Clone)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "../dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let server_app = Arc::new(ServerApp::new());

    let app = using_serve_dir(opt.clone(), server_app);

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(IP_ADDRESS).unwrap(),
        PORT.parse::<u16>().unwrap(),
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Unable to start server");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
}

fn using_serve_dir(opt: Opt, server_app: Arc<ServerApp>) -> Router {
    let closure = |req: Request<Body>| async move {
        match ServeDir::new(&opt.static_dir).oneshot(req).await {
            Ok(res) => {
                let status = res.status();
                match status {
                    StatusCode::NOT_FOUND => {
                        let index_path = PathBuf::from(&opt.static_dir).join("index.html");
                        let index_content = match fs::read_to_string(index_path).await {
                            Err(_) => {
                                return Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body(boxed(Body::from("index file not found")))
                                    .unwrap()
                            }
                            Ok(index_content) => index_content,
                        };

                        Response::builder()
                            .status(StatusCode::OK)
                            .body(boxed(Body::from(index_content)))
                            .unwrap()
                    }
                    _ => res.map(boxed),
                }
            }
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(boxed(Body::from(format!("error: {err}"))))
                .expect("error response"),
        }
    };

    Router::new()
        .route("/websocket", get(websocket_handler))
        .route("/api/check_room_exist", post(check_room_exist_handler))
        .route("/api/check_user_exist", post(check_user_exist_handler))
        .with_state(server_app)
        .fallback_service(get(closure))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}
