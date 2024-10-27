mod account;
mod error;
mod formatter;
mod subscriber;

use actix_web::web::Json;
use actix_web::{middleware, post, App, HttpResponse, HttpServer};
use log::info;

use crate::account::RegisterRequest;
use crate::formatter::ErrorResponseFormatter;
use crate::subscriber::ErrorSubscriber;

#[post("/register")]
async fn register(payload: Json<RegisterRequest>) -> Result<HttpResponse, error::Ec> {
    account::register(payload)?;
    return Ok(HttpResponse::Ok().into());
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().format_timestamp(None).init();

    // Use a custom formatter to avoid that any error metadata such as location or name
    // is included in the error response for the user.
    errore::formatter!(ErrorResponseFormatter);

    // Optionally a user defined subscriber for errors can be used for logging/tracing purposes.
    errore::subscriber!(ErrorSubscriber);

    let listen = "127.0.0.1:8080";
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(register)
    });

    info!("Listen on http://{}", listen);
    Ok(server.bind(listen)?.run().await?)

    // curl --header "Content-Type: application/json" --request POST --data '{"email":"xyz","password":"xyz"}' http://localhost:8080/register
}
