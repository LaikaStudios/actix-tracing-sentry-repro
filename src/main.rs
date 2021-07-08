use actix_web::http::StatusCode;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_web_requestid::{RequestID, RequestIDService};
use std::io;
use tracing::{event, span, subscriber::set_global_default, Level, Subscriber};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(sentry::integrations::tracing::layer())
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "DEBUG");
    if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        if !dsn.trim().is_empty() {
            eprintln!("Sentry DSN is {}", dsn);
            std::env::set_var("RUST_BACKTRACE", "1");
        } else {
            eprintln!("Set the SENTRY_DSN env var to enable sentry support");
        }
    }
    let subscriber = get_subscriber("app".into(), "info".into());
    init_subscriber(subscriber);

    let _guard = sentry::init(());

    let addr = "0.0.0.0:7878";

    let server = HttpServer::new(move || {
        App::new()
            .wrap(RequestIDService::default())
            .wrap(TracingLogger)
            .wrap(sentry_actix::Sentry::new())
            .service(web::resource("/panic").to(sentry_trigger_panic))
            .service(web::resource("/err").to(sentry_trigger_server_err))
            .service(web::resource("/event").to(sentry_trigger_event))
    })
    .bind(addr)?;
    event!(Level::INFO, "Server running. Listening on {}", &addr);
    Ok(server.run().await?)
}

async fn sentry_trigger_panic(req_id: RequestID) -> Result<HttpResponse> {
    let _span = span!(
        Level::DEBUG,
        "smoke test: forcing a panic",
        req = req_id.get().as_str()
    )
    .entered();
    panic!("Smoke Test");
}

async fn sentry_trigger_server_err(req_id: RequestID) -> Result<HttpResponse> {
    let _span = span!(
        Level::DEBUG,
        "smoke test: forcing a 500",
        req = req_id.get().as_str()
    )
    .entered();
    Err(actix_web::error::InternalError::new(
        io::Error::new(io::ErrorKind::Other, "smoke test\n"),
        StatusCode::INTERNAL_SERVER_ERROR,
    )
    .into())
}

async fn sentry_trigger_event(req_id: RequestID) -> Result<HttpResponse> {
    let _span = span!(
        Level::DEBUG,
        "smoke test: forcing an event",
        req = req_id.get().as_str()
    )
    .entered();
    event!(Level::ERROR, "smoke test: tracing's `event!()` macro");
    Ok(HttpResponse::Ok().body("okay, but not really\n"))
}
