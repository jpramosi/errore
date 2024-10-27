mod account;
mod error;
mod formatter;
mod subscriber;

use axum::extract::{MatchedPath, Request};
use axum::{routing::post, Json, Router};
use tower_http::trace::TraceLayer;
use tracing::{debug_span, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::account::RegisterRequest;
use crate::formatter::ErrorResponseFormatter;
use crate::subscriber::TracingSubscriber;

async fn register(Json(payload): Json<RegisterRequest>) -> Result<(), error::Ec> {
    account::register(Json(payload))?;
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Use a custom formatter to avoid that any error metadata such as location or name
    // is included in the error response for the user.
    errore::formatter!(ErrorResponseFormatter);

    // Optionally a user defined subscriber for errors can be used for logging/tracing purposes.
    errore::subscriber!(TracingSubscriber);

    let app = Router::new().route("/register", post(register)).layer(
        TraceLayer::new_for_http()
            // Create our own span for the request and include the matched path. The matched
            // path is useful for figuring out which handler the request was routed to.
            .make_span_with(|req: &Request| {
                let method = req.method();
                let uri = req.uri();

                // axum automatically adds this extension.
                let matched_path = req
                    .extensions()
                    .get::<MatchedPath>()
                    .map(|matched_path| matched_path.as_str());

                debug_span!("request", %method, %uri, matched_path)
            })
            // By default `TraceLayer` will log 5xx responses but we're doing our specific
            // logging of errors so disable that
            .on_failure(()),
    );

    let listen = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    info!("Listen on http://{}", listen);
    axum::serve(listener, app).await

    // curl --header "Content-Type: application/json" --request POST --data '{"email":"xyz","password":"xyz"}' http://localhost:8080/register
}
