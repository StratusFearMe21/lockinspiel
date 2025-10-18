use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, connect_info::Connected},
    http::StatusCode,
    routing::get,
    serve::IncomingStream,
};
use color_eyre::eyre::{self, Context};
use tokio::{net::TcpListener, signal};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::instrument;

use crate::error::WithStatusCode;

mod error;
mod time_sync;

#[derive(Clone, Copy, Debug)]
pub struct TimestampConnectInfo(pub SystemTime);

impl Connected<IncomingStream<'_, TcpListener>> for TimestampConnectInfo {
    fn connect_info(_stream: IncomingStream<'_, TcpListener>) -> Self {
        Self(SystemTime::now())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    lockinspiel_common::install_init_boilerplate(None)?;

    let app = Router::new()
        .route("/", get(index))
        .route("/time_sync", get(time_handler))
        .layer(CatchPanicLayer::custom(error::PanicHandler));

    let listen_addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(listen_addr)
        .await
        .wrap_err_with(|| format!("Failed to open listener on {}", listen_addr))?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<TimestampConnectInfo>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .wrap_err("Failed to serve make service")
}

#[instrument]
async fn index() -> maud::Markup {
    maud::html! {
        h1 { "It's working" }
        p { "The server is up" }
    }
}

#[instrument]
async fn time_handler(
    ConnectInfo(info): ConnectInfo<TimestampConnectInfo>,
) -> Result<Body, error::Error> {
    Ok(Body::new(time_sync::TimeDataStream::new(
        info.0
            .duration_since(UNIX_EPOCH)
            .wrap_err("Failed to convert SystemTime to micros since Unix epoch")
            .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)?,
    )))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
