use std::{net::SocketAddr, num::NonZeroUsize};

use axum::{
    extract::{
        ws::{self, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::{delete, get, put},
    Form, Router,
};
use hyper::StatusCode;
use serde::Deserialize;
use song_director_htmx::errors::{AppError, InitError};
use tokio::sync::watch as watch_channel;
use tower_http::services::ServeDir;
use tracing::Instrument;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

type SectionTuple = (Option<char>, Option<NonZeroUsize>);

#[derive(Clone)]
struct AppState<'a> {
    tera: tera::Tera,
    section_tx: &'a watch_channel::Sender<SectionTuple>,
    section_rx: watch_channel::Receiver<SectionTuple>,
}

#[derive(Deserialize)]
struct SectionType {
    section_type: char,
}

#[derive(Deserialize)]
struct SectionNumber {
    section_number: NonZeroUsize,
}

#[tokio::main]
async fn main() -> Result<(), InitError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .init();

    let tera = tera::Tera::new("templates/**/*.html")?;
    let (section_tx, section_rx) = watch_channel::channel((None, None));
    let app_state = AppState {
        tera,
        section_tx: Box::leak(Box::new(section_tx)),
        section_rx,
    };

    let file_service = ServeDir::new("public").precompressed_br();
    let app = Router::new()
        .route("/", get(controller))
        .route("/view", get(view))
        .route("/section/type", put(set_section_type))
        .route("/section/number", put(set_section_number))
        .route("/section", delete(clear_section))
        .route("/section", get(section_ws_handler))
        .with_state(app_state)
        .fallback_service(file_service);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!(
        "song director server v{} listening on http://{}",
        env!("CARGO_PKG_VERSION"),
        &addr
    );
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    return Ok(());
}

fn section_segments_to_string(segments: &SectionTuple) -> String {
    if let Some(sec) = segments.0 {
        if let Some(num) = segments.1 {
            format!("{}{}", sec, num)
        } else {
            sec.to_string()
        }
    } else {
        // Zero-width space so vertical space in UI is maintained
        "\u{200b}".to_string()
    }
}

async fn controller(
    State(AppState {
        tera, section_rx, ..
    }): State<AppState<'_>>,
) -> Result<Html<String>, AppError> {
    let section = section_rx.borrow();
    let mut context = tera::Context::new();
    context.insert("song_section", &section_segments_to_string(&section));

    return Ok(Html(tera.render("controller.html", &context)?));
}

async fn view(
    State(AppState {
        tera, section_rx, ..
    }): State<AppState<'_>>,
) -> Result<Html<String>, AppError> {
    let section = section_rx.borrow();
    let mut context = tera::Context::new();
    context.insert("song_section", &section_segments_to_string(&section));

    return Ok(Html(tera.render("viewer.html", &context)?));
}

async fn set_section_type(
    State(AppState { section_tx, .. }): State<AppState<'_>>,
    Form(SectionType { section_type }): Form<SectionType>,
) -> StatusCode {
    tracing::debug!("Setting section type to {}", section_type);
    section_tx.send_replace((Some(section_type), None));

    return StatusCode::NO_CONTENT;
}

async fn set_section_number(
    State(AppState { section_tx, .. }): State<AppState<'_>>,
    Form(SectionNumber { section_number }): Form<SectionNumber>,
) -> StatusCode {
    tracing::debug!("Setting section number to {}", section_number);
    section_tx.send_modify(|val| val.1 = Some(section_number));

    return StatusCode::NO_CONTENT;
}

async fn clear_section(State(AppState { section_tx, .. }): State<AppState<'_>>) -> StatusCode {
    tracing::debug!("Clearing section");
    section_tx.send_replace((None, None));

    return StatusCode::NO_CONTENT;
}

async fn section_ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(AppState {
        tera, section_rx, ..
    }): State<AppState<'_>>,
) -> impl IntoResponse {
    return ws.on_upgrade(move |socket| {
        section_socket(socket, tera, section_rx).instrument(tracing::info_span!(
            "section_socket",
            client_addr = addr.to_string()
        ))
    });
}

async fn section_socket(
    mut socket: WebSocket,
    tera: tera::Tera,
    mut section_rx: watch_channel::Receiver<SectionTuple>,
) {
    tracing::info!("Socket connection established");
    loop {
        tokio::select! {
            biased;
            Some(Ok(ws::Message::Close(_))) = socket.recv() => {
                tracing::info!("Client closed socket");
                return;
            },
            changed = section_rx.changed() =>
            if changed.is_ok() {
                let mut context = tera::Context::new();
                context.insert(
                    "song_section",
                    &section_segments_to_string(&section_rx.borrow()),
                );
                match tera.render("fragments/section-display.html", &context) {
                    Ok(message_text) => {
                        tracing::debug!("Sending {}", message_text);
                        if let Err(err) = socket.send(ws::Message::Text(message_text)).await {
                            tracing::warn!("Error sending message: {}", err);
                            tracing::info!("Closing socket");
                            return;
                        }
                    }
                    Err(err) => {
                        tracing::error!("Error rendering template: {}", err);
                        tracing::info!("Closing socket");
                        return;
                    }
                };
            } else {
                // Channel has closed. Should never actually happen
                tracing::error!("Section channel closed unexpectedly");
                tracing::info!("Closing socket");
                let _ = socket.close().await;
                return;
            }
        }
    }
}
